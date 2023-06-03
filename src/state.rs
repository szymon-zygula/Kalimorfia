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
            selector: Self::new_selector(entity_manager, selected_aggregate_id),
            selected_aggregate_id,
        }
    }

    fn new_selector(
        entity_manager: &'a RefCell<EntityManager<'gl>>,
        selected_aggregate_id: usize,
    ) -> Selector<'a> {
        Selector::new(
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
                let deleted_name = entity_manager.borrow().get_entity(id).name();
                let removal_result = entity_manager.borrow_mut().remove_entity(id);

                removal_result.map(|blocker| {
                    format!(
                        "Deletion of {deleted_name} blocked by {}",
                        entity_manager.borrow().get_entity(blocker).name()
                    )
                })
            },
        )
    }

    pub fn reset(
        &mut self,
        gl: &'gl glow::Context,
        entity_manager: &'a RefCell<EntityManager<'gl>>,
        shader_manager: Rc<ShaderManager<'gl>>,
    ) {
        self.selected_aggregate_id =
            entity_manager
                .borrow_mut()
                .add_entity(Box::new(Aggregate::new(
                    gl,
                    &mut ExactNameRepository::new(),
                    shader_manager,
                )));

        let old_res = self.camera.resolution;
        self.camera = Camera::new();
        self.camera.resolution = old_res;
        self.name_repo = Rc::new(RefCell::new(UniqueNameRepository::new()));
        self.selector = Self::new_selector(entity_manager, self.selected_aggregate_id);
    }
}
