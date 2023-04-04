use kalimorfia::{
    camera::Camera,
    entities::{aggregate::Aggregate, cursor::ScreenCursor, manager::EntityManager},
    render::shader_manager::ShaderManager,
    repositories::{ExactNameRepository, NameRepository, UniqueNameRepository},
    ui::selector::Selector,
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
        entity_manager: &'a RefCell<EntityManager<'gl>>,
        shader_manager: Rc<ShaderManager<'gl>>,
    ) -> Self {
        let selected_aggregate_id =
            entity_manager
                .borrow_mut()
                .add_entity(Box::new(Aggregate::new(
                    gl,
                    &mut ExactNameRepository::new(),
                    Rc::clone(&shader_manager),
                )));

        State {
            camera: Camera::new(),
            cursor: ScreenCursor::new(gl, Camera::new(), Rc::clone(&shader_manager)),
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
}
