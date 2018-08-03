
#[derive(Clone, Serialize, Deserialize)]
pub struct Effect {
}


pub trait HaveEffect {
    fn effect(&self) -> &Effect;
    fn effect_mut(&mut self) -> &mut Effect;
}
