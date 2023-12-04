use crate::app::event::{Event, StateChange, UserAction};
use crate::app::state::{Consumed, SessionState};
use crate::app::{AeonError, DynError};
use crate::sketchbook::{RegulationsState, VarId};
use serde_json::json;

/// TODO: expand this.
/// List of events we will need, with their path, payload format, and path to the corresponding
/// front-end event. Payloads are single string values,
/// or a json object with multiple fields.
/// ["variable", "add"]; payload = var_id; `add-variable`
/// ["variable", "remove"]; payload = var_id, `remove-variable`
/// ["variable", "set_name"]; payload = {"id": var_id, "new_name": new_name}; `set-name`
/// ["variable", "set_id"],; payload = {"original_id": original_id, "new_id": new_id}; `set-id`

/// Functionality and shorthands related for the `SessionState` trait.
impl RegulationsState {
    /// Shorthand to get and clone a payload of an action. Errors if payload is empty.
    /// The `component` specifies which part of the state should be mentioned in the error.
    /// In future we may consider moving this elsewhere.
    fn clone_payload(action: &UserAction, component: &str) -> Result<String, DynError> {
        let var_name = action
            .event
            .payload
            .clone()
            .ok_or(format!(
                "Event to `{component}` cannot carry empty payload."
            ))?
            .clone();
        Ok(var_name)
    }

    /// Shorthand to get and clone a full path of an action.
    /// In future we may consider moving this elsewhere.
    fn clone_path(action: &UserAction) -> Vec<String> {
        action.event.full_path.clone()
    }

    /// Shorthand to assert that path has given length and return typesafe `DynError` otherwise.
    /// The `component` specifies which component of the state should be mentioned in the error.
    fn assert_path_length(path: &[&str], length: usize, component: &str) -> Result<(), DynError> {
        if path.len() != length {
            return AeonError::throw(format!("`{component}` cannot consume a path `{:?}`.", path));
        }
        Ok(())
    }

    /// Consume events related to `variables` component of this `RegulationsState`. Currently
    /// involves:
    ///     > `add`, payload = var_id
    ///     > `remove`, payload = var_id
    ///     > `set_name`, payload = json{"id": var_id, "new_name": new_name}
    ///     > `set_id`, payload = json{"original_id": original_id, "new_id": new_id}
    fn consume_variable_event(
        &mut self,
        path: &[&str],
        action: &UserAction,
    ) -> Result<Consumed, DynError> {
        let component_name = "RegulationState/variable";
        println!("Consuming action {:?} at {component_name}", action);
        Self::assert_path_length(path, 1, component_name)?;

        match path.first() {
            Some(&"add") => {
                let var_name = Self::clone_payload(action, component_name)?;
                self.add_var_by_str(var_name.as_str(), var_name.as_str())?;

                let state_change =
                    StateChange::from(Event::build(&["add-variable"], Some(var_name.as_str())));

                let mut reverse_path = Self::clone_path(action);
                reverse_path.remove(reverse_path.len() - 1);
                reverse_path.push("remove".to_string());
                let reverse_path: Vec<&str> = reverse_path.iter().map(|s| s.as_str()).collect();
                let reverse_action =
                    UserAction::from(Event::build(&reverse_path, Some(var_name.as_str())));

                Ok(Consumed::Reversible(
                    state_change,
                    (action.clone(), reverse_action),
                ))
            }
            Some(&"remove") => {
                let var_name = Self::clone_payload(action, component_name)?;
                self.remove_var_by_str(var_name.as_str())?;

                let state_change =
                    StateChange::from(Event::build(&["remove-variable"], Some(var_name.as_str())));
                Ok(Consumed::Irreversible(state_change))
            }
            Some(&"set_name") => {
                let payload = Self::clone_payload(action, component_name)?;
                let payload_json: serde_json::Value = serde_json::from_str(payload.as_str())?;
                let var_id = self.get_var_id(
                    payload_json["id"]
                        .as_str()
                        .ok_or(format!("Payload {payload} is invalid"))?
                        .trim_matches('\"'),
                )?;
                let new_name = payload_json["new_name"]
                    .as_str()
                    .ok_or(format!("Payload {payload} is invalid"))?
                    .trim_matches('\"');
                let original_name = self.get_var_name(&var_id)?.to_string();

                self.set_var_name(&var_id, new_name)?;
                let state_change = StateChange::from(Event::build(&["set-name"], Some(&payload)));

                let mut reverse_action = action.clone();
                reverse_action.event.payload = Some(
                    json!({
                        "id": var_id.as_str(),
                        "new_name": original_name,
                    })
                    .to_string(),
                );

                Ok(Consumed::Reversible(
                    state_change,
                    (action.clone(), reverse_action),
                ))
            }
            Some(&"set_id") => {
                let payload = Self::clone_payload(action, component_name)?;
                let payload_json: serde_json::Value = serde_json::from_str(payload.as_str())?;
                let original_id = self.get_var_id(
                    payload_json["original_id"]
                        .as_str()
                        .ok_or(format!("Payload {payload} is invalid"))?
                        .trim_matches('\"'),
                )?;
                let new_id = VarId::new(
                    payload_json["new_id"]
                        .as_str()
                        .ok_or(format!("Payload {payload} is invalid"))?
                        .trim_matches('\"'),
                )?;

                self.set_var_id(&original_id, new_id.clone())?;
                let state_change = StateChange::from(Event::build(&["set-id"], Some(&payload)));

                let mut reverse_action = action.clone();
                reverse_action.event.payload = Some(
                    json!({
                        "original_id": new_id.as_str(),
                        "new_id": original_id.as_str(),
                    })
                    .to_string(),
                );

                Ok(Consumed::Reversible(
                    state_change,
                    (action.clone(), reverse_action),
                ))
            }
            _ => AeonError::throw(format!(
                "`RegulationState/variable` cannot consume a path `{:?}`.",
                path
            )),
        }
    }

    /// Consume events related to `regulations` component of this `RegulationsState`.
    fn consume_regulation_event(
        &mut self,
        path: &[&str],
        action: &UserAction,
    ) -> Result<Consumed, DynError> {
        let component_name = "RegulationState/regulation";
        println!("Consuming action {:?} at {component_name}", action);
        Self::assert_path_length(path, 1, component_name)?;

        match path.first() {
            Some(&"add") => {
                todo!()
            }
            Some(&"remove") => {
                todo!()
            }
            _ => AeonError::throw(format!(
                "`RegulationState/regulation` cannot consume a path `{:?}`.",
                path
            )),
        }
    }

    /// Consume events related to `layouts` component of this `RegulationsState`.
    fn consume_layout_event(
        &mut self,
        path: &[&str],
        action: &UserAction,
    ) -> Result<Consumed, DynError> {
        let component_name = "RegulationState/layout";
        println!("Consuming action {:?} at {component_name}", action);
        Self::assert_path_length(path, 1, component_name)?;

        match path.first() {
            Some(&"add") => {
                todo!()
            }
            Some(&"remove") => {
                todo!()
            }
            _ => AeonError::throw(format!(
                "`RegulationState/layout` cannot consume a path `{:?}`.",
                path
            )),
        }
    }
}

impl SessionState for RegulationsState {
    fn consume_event(&mut self, path: &[&str], action: &UserAction) -> Result<Consumed, DynError> {
        Self::assert_path_length(path, 2, "RegulationState")?;

        match path.first() {
            Some(&"variable") => self.consume_variable_event(&path[1..], action),
            Some(&"regulation") => self.consume_regulation_event(&path[1..], action),
            Some(&"layout") => self.consume_layout_event(&path[1..], action),
            _ => AeonError::throw(format!(
                "`RegulationState` cannot consume a path `{:?}`.",
                path
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::event::{Event, UserAction};
    use crate::app::state::{Consumed, SessionState};
    use crate::sketchbook::{RegulationsState, VarId};
    use serde_json::json;

    #[test]
    fn test_add_var_event() {
        let mut reg_state = RegulationsState::new();
        let var_id_a = reg_state.generate_var_id("a");
        reg_state.add_var(var_id_a, "a").unwrap();
        let reg_state_orig = reg_state.clone();
        assert_eq!(reg_state.num_vars(), 1);

        // test variable add event
        let event = Event::build(&["regulations_state", "variable", "add"], Some("b"));
        let action = UserAction::from(event);
        let result = reg_state
            .consume_event(&["variable", "add"], &action)
            .unwrap();

        // check var was added
        assert_eq!(reg_state.num_vars(), 2);
        assert_eq!(reg_state.get_var_id("b").unwrap(), VarId::new("b").unwrap());

        // assert that the reverse action is correct
        assert!(matches!(result, Consumed::Reversible(..)));
        if let Consumed::Reversible(_, (_, reverse)) = result {
            reg_state
                .consume_event(&["variable", "remove"], &reverse)
                .unwrap();
            assert_eq!(reg_state, reg_state_orig);
        }
    }

    #[test]
    fn test_remove_var_event() {
        let mut reg_state = RegulationsState::new();
        let var_id_a = reg_state.generate_var_id("a");
        reg_state.add_var(var_id_a, "a").unwrap();
        reg_state.add_regulation_by_str("a -> a").unwrap();
        assert_eq!(reg_state.num_vars(), 1);
        assert_eq!(reg_state.num_regulations(), 1);

        // test variable remove event
        let event = Event::build(&["regulations_state", "variable", "remove"], Some("a"));
        let action = UserAction::from(event);
        let result = reg_state
            .consume_event(&["variable", "remove"], &action)
            .unwrap();

        // check var was removed
        assert_eq!(reg_state.num_vars(), 0);
        assert_eq!(reg_state.num_regulations(), 0);

        // assert that it is irreversible (at the moment)
        assert!(matches!(result, Consumed::Irreversible(..)));
    }

    #[test]
    fn test_set_var_name_event() {
        let mut reg_state = RegulationsState::new();
        let var_id = reg_state.generate_var_id("a");
        let original_name = "a_name";
        let new_name = "new_name";
        reg_state.add_var(var_id.clone(), original_name).unwrap();
        let reg_state_orig = reg_state.clone();
        assert_eq!(reg_state.get_var_name(&var_id).unwrap(), original_name);

        // test variable rename event
        let payload = json!({
            "id": var_id.as_str(),
            "new_name": new_name,
        })
        .to_string();
        let event = Event::build(
            &["regulations_state", "variable", "set_name"],
            Some(payload.as_str()),
        );
        let action = UserAction::from(event);
        let result = reg_state
            .consume_event(&["variable", "set_name"], &action)
            .unwrap();

        // check var was renamed
        assert_eq!(reg_state.get_var_name(&var_id).unwrap(), new_name);

        // assert that the reverse action is correct
        assert!(matches!(result, Consumed::Reversible(..)));
        if let Consumed::Reversible(_, (_, reverse)) = result {
            reg_state
                .consume_event(&["variable", "set_name"], &reverse)
                .unwrap();
            assert_eq!(reg_state.get_var_name(&var_id).unwrap(), original_name);
            assert_eq!(reg_state, reg_state_orig);
        }
    }

    #[test]
    fn test_set_var_id_event() {
        let mut reg_state = RegulationsState::new();
        let var_id = reg_state.generate_var_id("a");
        reg_state.add_var(var_id.clone(), "a_name").unwrap();
        let reg_state_orig = reg_state.clone();

        // test id change event
        let new_id = reg_state.generate_var_id("b");
        let payload = json!({
            "original_id": var_id.as_str(),
            "new_id": new_id.as_str(),
        })
        .to_string();
        let event = Event::build(
            &["regulations_state", "variable", "set_id"],
            Some(payload.as_str()),
        );
        let action = UserAction::from(event);
        let result = reg_state
            .consume_event(&["variable", "set_id"], &action)
            .unwrap();

        // check id changed
        assert!(!reg_state.is_valid_var_id(&var_id));
        assert!(reg_state.is_valid_var_id(&new_id));

        // assert that the reverse action is correct
        assert!(matches!(result, Consumed::Reversible(..)));
        if let Consumed::Reversible(_, (_, reverse)) = result {
            reg_state
                .consume_event(&["variable", "set_id"], &reverse)
                .unwrap();
            assert_eq!(reg_state, reg_state_orig);
        }
    }

    #[test]
    fn test_invalid_var_events() {
        let mut reg_state = RegulationsState::new();
        let var_id = reg_state.generate_var_id("a");
        reg_state.add_var(var_id.clone(), "a-name").unwrap();
        let reg_state_orig = reg_state.clone();

        // adding variable `a` again
        let event = Event::build(&["regulations_state", "variable", "add"], Some("a"));
        let action = UserAction::from(event);
        assert!(reg_state
            .consume_event(&["variable", "add"], &action)
            .is_err());
        assert_eq!(reg_state, reg_state_orig);

        // removing variable with wrong id
        let event = Event::build(&["regulations_state", "variable", "remove"], Some("b"));
        let action = UserAction::from(event);
        assert!(reg_state
            .consume_event(&["variable", "remove"], &action)
            .is_err());
        assert_eq!(reg_state, reg_state_orig);

        // variable rename event with wrong id
        let payload = json!({"id": "b","new_name": "x",}).to_string();
        let event = Event::build(
            &["regulations_state", "variable", "set_name"],
            Some(payload.as_str()),
        );
        let action = UserAction::from(event);
        assert!(reg_state
            .consume_event(&["variable", "set_name"], &action)
            .is_err());
        assert_eq!(reg_state, reg_state_orig);
    }
}
