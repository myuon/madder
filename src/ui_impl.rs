
use madder_core::*;
use widget::*;

pub struct ComponentRenderer {
    pub object: BoxObject,
    pub object_type: ComponentType,
}

impl AsRef<BoxObject> for ComponentRenderer {
    fn as_ref(&self) -> &BoxObject {
        &self.object
    }
}

