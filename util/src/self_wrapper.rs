#[derive(Clone)]
pub struct Model<T>(*mut T);

impl<T> Model<T> {
    pub fn new(self_: &mut T) -> Self {
        Model(self_ as *mut T)
    }
}

impl<T> AsRef<T> for Model<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.0.as_ref().expect("***Nullpointer exception: &Self***") }
    }
}

impl<T> AsMut<T> for Model<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut().expect("***Nullpointer exception: &mut Self***") }
    }
}

