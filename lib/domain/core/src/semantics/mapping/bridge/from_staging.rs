use crate::interop::staging::StgSrCodeExploded;

use crate::semantics::mapping::CodeElement;

impl From<StgSrCodeExploded> for CodeElement {
    fn from(v: StgSrCodeExploded) -> Self {
        CodeElement::new(
            CodeElement::id_for(
                &v.sr_id,
                v.system.as_deref(),
                v.code.as_deref(),
                v.display.as_deref(),
            ),
            v.system,
            v.code,
            v.display,
        )
    }
}

impl From<&StgSrCodeExploded> for CodeElement {
    fn from(v: &StgSrCodeExploded) -> Self {
        CodeElement::new(
            CodeElement::id_for(
                &v.sr_id,
                v.system.as_deref(),
                v.code.as_deref(),
                v.display.as_deref(),
            ),
            v.system.clone(),
            v.code.clone(),
            v.display.clone(),
        )
    }
}
