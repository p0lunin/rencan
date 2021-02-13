use vulkano::instance::QueueFamily;

pub trait QueueFamilyExt {
    fn proof_support_graphics(&self) -> Option<ProofSupportGraphics>;
}

impl<'a> QueueFamilyExt for QueueFamily<'a> {
    fn proof_support_graphics(&self) -> Option<ProofSupportGraphics<'a>> {
        match self.supports_graphics() {
            true => Some(ProofSupportGraphics(self.clone())),
            false => None,
        }
    }
}

pub struct ProofSupportGraphics<'a>(QueueFamily<'a>);

impl<'a> ProofSupportGraphics<'a> {
    pub fn into_inner(self) -> QueueFamily<'a> {
        self.0
    }
}
