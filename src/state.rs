use kalimorfia::{
    camera::Camera,
    entities::{
        aggregate::Aggregate, cursor::ScreenCursor, entity::ReferentialSceneEntity,
        manager::EntityManager,
    },
    repositories::ExactNameRepository,
    repositories::NameRepository,
    repositories::UniqueNameRepository,
    ui::selector::Selector,
    window::Window,
};
use std::{cell::RefCell, rc::Rc};

pub struct State<'gl, 'a> {
    pub cursor: ScreenCursor<'gl>,
    pub camera: Camera,
    pub selector: Selector<'a>,
    pub name_repo: Rc<RefCell<dyn NameRepository>>,
    pub selected_aggregate_id: usize,
}

impl<'gl, 'a> State<'gl, 'a> {
    pub fn new(
        gl: &'gl glow::Context,
        window: &Window,
        entity_manager: &'a RefCell<EntityManager<'gl>>,
    ) -> Self {
        let selected_aggregate_id =
            entity_manager
                .borrow_mut()
                .add_entity(Box::new(Aggregate::new(
                    gl,
                    &mut ExactNameRepository::new(),
                )));

        State {
            camera: Camera::new(),
            cursor: ScreenCursor::new(gl, Camera::new(), window.size()),
            name_repo: Rc::new(RefCell::new(UniqueNameRepository::new())),
            selector: Selector::new(
                move |id| {
                    entity_manager
                        .borrow_mut()
                        .subscribe(selected_aggregate_id, id);
                },
                move |id| {
                    entity_manager
                        .borrow_mut()
                        .unsubscribe(selected_aggregate_id, id);
                },
                move |id| {
                    entity_manager.borrow_mut().remove_entity(id);
                },
            ),
            selected_aggregate_id,
        }
    }

    pub fn add_entity(
        &mut self,
        entity: Box<dyn ReferentialSceneEntity<'gl> + 'gl>,
        entity_manager: &mut EntityManager<'gl>,
    ) {
        let id = entity_manager.add_entity(entity);
        self.selector.add_selectable(id);
    }
}
