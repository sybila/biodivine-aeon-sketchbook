use crate::sketchbook::layout::{Layout, LayoutId};
use crate::sketchbook::{Regulation, VarId, Variable};
use std::collections::{HashMap, HashSet};

/// **(internal)** Methods for safely constructing or editing instances of `RegulationsState`.
mod _impl_editing;
/// **(internal)** Implementation of the safe identifier generating.
mod _impl_id_generating;
/// **(internal)** Methods for observing instances of `RegulationsState` (various getters, etc.).
mod _impl_observing;
/// **(internal)** Minor traits like [PartialEq] or [Display].
mod _impl_other_traits;
/// **(internal)** Methods for converting between `RegulationsState` and `RegulatoryGraph`.
mod _impl_reg_graph_conversion;
/// **(internal)** Implementation of serialization traits [Serialize] and [Deserialize].
mod _impl_serde;
/// **(internal)** Implementation of the [SessionState] trait.
mod _impl_session_state;

/// Object representing the state of the Regulations editor.
///
/// Holds similar information as `RegulatoryGraph` in `lib-param-bn`, but is suitable for
/// editing. Further, the information regarding the layout is carried.
#[derive(Clone, Debug)]
pub struct RegulationsState {
    variables: HashMap<VarId, Variable>,
    regulations: HashSet<Regulation>,
    layouts: HashMap<LayoutId, Layout>,
}
