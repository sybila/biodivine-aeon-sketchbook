use crate::app::event::Event;
use crate::app::state::{Consumed, SessionState};
use crate::sketchbook::data_structs::{
    ChangeArgEssentialData, ChangeArgMonotoneData, LayoutData, LayoutNodeData, RegulationData,
    UninterpretedFnData, VariableData,
};
use crate::sketchbook::layout::NodePosition;
use crate::sketchbook::{Essentiality, ModelState, Monotonicity, VarId};

/// Check that after applying the reverse event of `result` to the `model` with relative
/// path `at_path`, we receive precisely `model_orig`.
fn check_reverse(
    mut model: ModelState,
    model_orig: ModelState,
    result: Consumed,
    at_path: &[&str],
) {
    // assert that the reverse event is correct
    match result {
        Consumed::Reversible {
            perform_reverse: (_, reverse),
            ..
        } => {
            model.perform_event(&reverse, &at_path).unwrap();
            assert_eq!(model, model_orig);
        }
        _ => panic!(),
    }
}

#[test]
fn test_add_var_event() {
    let mut model = ModelState::new();
    let var_id_a = model.generate_var_id("a");
    model.add_var(var_id_a, "a").unwrap();
    let model_orig = model.clone();
    assert_eq!(model.num_vars(), 1);

    // test variable add event
    let payload = VariableData::new("b", "b", "").to_string();
    let full_path = ["model", "variable", "add"];
    let event = Event::build(&full_path, Some(payload.as_str()));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check var was added
    assert_eq!(model.num_vars(), 2);
    assert_eq!(model.get_var_id("b").unwrap(), VarId::new("b").unwrap());
    check_reverse(model, model_orig, result, &["variable", "b", "remove"]);
}

#[test]
/// Test removing a variable that has no regulations and its node is in default position.
fn test_remove_var_event_simple() {
    let variables = vec![("a", "a_name"), ("b", "b_name")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    let model_orig = model.clone();

    // test variable remove event
    let full_path = ["model", "variable", "a", "remove"];
    let event = Event::build(&full_path, None);
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check var was removed - result should be a simple `Consumed::Reversible` object
    assert_eq!(model.num_vars(), 1);
    check_reverse(model, model_orig, result, &["variable", "add"]);
}

#[test]
/// Test removing a variable that has regulations and its node is in non-default position.
fn test_remove_var_event_complex() {
    let mut model = ModelState::new();
    let var_id_a = model.generate_var_id("a");
    model.add_var(var_id_a.clone(), "a").unwrap();
    model.add_var_by_str("b", "b").unwrap();
    model
        .add_multiple_regulations(vec!["a -> a", "a -> b", "b -> b"])
        .unwrap();
    model
        .update_node_position(&ModelState::get_default_layout_id(), &var_id_a, 1., 1.)
        .unwrap();

    // expected result
    let mut model_expected = ModelState::new();
    model_expected.add_var_by_str("b", "b").unwrap();
    model_expected.add_regulation_by_str("b -> b").unwrap();

    // test variable remove event
    let full_path = ["model", "variable", "a", "remove"];
    let event = Event::build(&full_path, None);
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // result should be a `Consumed::Restart` object with a vector of events which should simulate the individual
    // steps of the variable's removal
    if let Consumed::Restart(mut sub_events) = result {
        // there should be 2 events for regulations removal, 1 for re-position, and final one for var removal
        assert_eq!(sub_events.len(), 4);
        sub_events.reverse();
        for e in sub_events {
            let mut full_path = e.path.clone();
            full_path.remove(0);
            let full_path_str: Vec<&str> = full_path.iter().map(|s| s.as_str()).collect();
            println!("{:?}", e);
            model.perform_event(&e, &full_path_str).unwrap();
        }
        assert_eq!(model, model_expected);
    } else {
        unreachable!();
    }
}

#[test]
fn test_set_var_name_event() {
    let mut model = ModelState::new();
    let var_id = model.generate_var_id("a");
    let original_name = "a_name";
    model.add_var(var_id.clone(), original_name).unwrap();
    let new_name = "new_name";
    let model_orig = model.clone();
    assert_eq!(model.get_var_name(&var_id).unwrap(), original_name);

    // test variable rename event
    let full_path = ["model", "variable", var_id.as_str(), "set_name"];
    let event = Event::build(&full_path, Some(new_name));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check var was renamed
    assert_eq!(model.get_var_name(&var_id).unwrap(), new_name);
    check_reverse(model, model_orig, result, &full_path[1..]);
}

#[test]
fn test_set_var_id_event() {
    let mut model = ModelState::new();
    let var_id = model.generate_var_id("a");
    model.add_var(var_id.clone(), "a_name").unwrap();
    let model_orig = model.clone();

    // test id change event
    let new_id = model.generate_var_id("b");
    let full_path = ["model", "variable", var_id.as_str(), "set_id"];
    let event = Event::build(&full_path, Some(new_id.as_str()));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check id changed
    assert!(!model.is_valid_var_id(&var_id));
    assert!(model.is_valid_var_id(&new_id));
    check_reverse(
        model,
        model_orig,
        result,
        &["variable", new_id.as_str(), "set_id"],
    );
}

#[test]
fn test_set_update_fn_event() {
    let variables = vec![("a", "a_name"), ("b", "b_name")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    let model_orig = model.clone();
    let expression = "a => b";
    let var_a = model.get_var_id("a").unwrap();

    // test update fn modification event
    let full_path = ["model", "variable", var_a.as_str(), "set_update_fn"];
    let event = Event::build(&full_path, Some(expression));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();
    assert_eq!(model.get_update_fn_string(&var_a).unwrap(), expression);
    check_reverse(
        model,
        model_orig,
        result,
        &["variable", var_a.as_str(), "set_update_fn"],
    );
}

#[test]
fn test_invalid_var_events() {
    let mut model = ModelState::new();
    let var_id = model.generate_var_id("a");
    model.add_var(var_id.clone(), "a-name").unwrap();
    let model_orig = model.clone();

    // adding variable `a` again
    let full_path = ["model", "variable", "add"];
    let event = Event::build(&full_path, Some("a"));
    assert!(model.perform_event(&event, &full_path[1..]).is_err());
    assert_eq!(model, model_orig);

    // removing variable with wrong id
    let full_path = ["model", "variable", "b", "remove"];
    let event = Event::build(&full_path, None);
    assert!(model.perform_event(&event, &full_path[1..]).is_err());
    assert_eq!(model, model_orig);

    // variable rename event with wrong id
    let full_path = ["model", "variable", "b", "set_name"];
    let event = Event::build(&full_path, Some("new_name"));
    assert!(model.perform_event(&event, &full_path[1..]).is_err());
    assert_eq!(model, model_orig);
}

#[test]
fn test_add_reg_event() {
    let variables = vec![("a", "a"), ("b", "b")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    let model_orig = model.clone();

    // test regulation add event
    let full_path = ["model", "regulation", "add"];
    let regulation_data = RegulationData::try_from_reg_str("a -> b").unwrap();
    let event = Event::build(&full_path, Some(&regulation_data.to_string()));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    assert_eq!(model.num_regulations(), 1);
    check_reverse(
        model,
        model_orig,
        result,
        &["regulation", "a", "b", "remove"],
    );
}

#[test]
fn test_change_reg_sign_event() {
    let variables = vec![("a", "a_name"), ("b", "b_name")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    let regulations = vec!["a -> a", "a -> b", "b -> a"];
    model.add_multiple_regulations(regulations).unwrap();
    let model_orig = model.clone();

    // test event for changing regulation's sign
    let full_path = ["model", "regulation", "a", "b", "set_sign"];
    let new_sign = serde_json::to_string(&Monotonicity::Inhibition).unwrap();
    let event = Event::build(&full_path, Some(&new_sign));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    check_reverse(
        model,
        model_orig,
        result,
        &["regulation", "a", "b", "set_sign"],
    );
}

#[test]
fn test_change_reg_essentiality_event() {
    let variables = vec![("a", "a_name"), ("b", "b_name")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    let regulations = vec!["a -> a", "a -> b", "b -> a"];
    model.add_multiple_regulations(regulations).unwrap();
    let model_orig = model.clone();

    // test event for changing regulation's essentiality
    let full_path = ["model", "regulation", "a", "b", "set_essentiality"];
    let new_essentiality = serde_json::to_string(&Essentiality::False).unwrap();
    let event = Event::build(&full_path, Some(&new_essentiality));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    check_reverse(
        model,
        model_orig,
        result,
        &["regulation", "a", "b", "set_essentiality"],
    );
}

#[test]
fn test_remove_reg_event() {
    let variables = vec![("a", "a_name"), ("b", "b_name")];
    let mut model = ModelState::new_from_vars(variables).unwrap();
    model.add_regulation_by_str("a -> b").unwrap();
    let model_orig = model.clone();

    // test regulation add event
    let full_path = ["model", "regulation", "a", "b", "remove"];
    let event = Event::build(&full_path, None);
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    assert_eq!(model.num_regulations(), 0);
    check_reverse(model, model_orig, result, &["regulation", "add"]);
}

#[test]
fn test_change_position_event() {
    let mut model = ModelState::new();
    let layout_id = ModelState::get_default_layout_id();
    let var_id = model.generate_var_id("a");
    model.add_var(var_id.clone(), "a_name").unwrap();
    let model_orig = model.clone();

    // test position change event
    let payload = LayoutNodeData::new(layout_id.as_str(), var_id.as_str(), 2.5, 0.4).to_string();
    let full_path = ["model", "layout", layout_id.as_str(), "update_position"];
    let event = Event::build(&full_path, Some(payload.as_str()));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check position changed
    assert_eq!(
        model.get_node_position(&layout_id, &var_id).unwrap(),
        &NodePosition(2.5, 0.4)
    );

    check_reverse(model, model_orig, result, &full_path[1..]);
}

#[test]
fn test_change_fn_arg_monotonicity_event() {
    let mut model = ModelState::new();
    let f = model.generate_uninterpreted_fn_id("f");
    model.add_new_uninterpreted_fn(f.clone(), "f", 2).unwrap();
    let model_orig = model.clone();

    // test event for changing uninterpreted fn's monotonicity
    let full_path = ["model", "uninterpreted_fn", f.as_str(), "set_monotonicity"];
    let change_data = ChangeArgMonotoneData::new(1, Monotonicity::Dual).to_string();
    let event = Event::build(&full_path, Some(&change_data));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check if changed correctly
    assert_eq!(
        Monotonicity::Dual,
        *model.get_uninterpreted_fn(&f).unwrap().get_monotonic(1)
    );

    check_reverse(
        model,
        model_orig,
        result,
        &["uninterpreted_fn", f.as_str(), "set_monotonicity"],
    );
}

#[test]
fn test_change_fn_arg_essentiality_event() {
    let mut model = ModelState::new();
    let f = model.generate_uninterpreted_fn_id("f");
    model.add_new_uninterpreted_fn(f.clone(), "f", 2).unwrap();
    let model_orig = model.clone();

    // test event for changing uninterpreted fn's expression
    let full_path = ["model", "uninterpreted_fn", f.as_str(), "set_essentiality"];
    let change_data = ChangeArgEssentialData::new(1, Essentiality::True).to_string();
    let event = Event::build(&full_path, Some(&change_data));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check if changed correctly
    assert_eq!(
        Essentiality::True,
        *model.get_uninterpreted_fn(&f).unwrap().get_essential(1)
    );

    check_reverse(
        model,
        model_orig,
        result,
        &["uninterpreted_fn", f.as_str(), "set_essentiality"],
    );
}

#[test]
fn test_change_fn_expression_event() {
    let mut model = ModelState::new();
    let f = model.generate_uninterpreted_fn_id("f");
    model.add_new_uninterpreted_fn(f.clone(), "f", 2).unwrap();
    let model_orig = model.clone();

    // test event for changing uninterpreted fn's expression
    let expression = "var0 | var1";
    let full_path = ["model", "uninterpreted_fn", f.as_str(), "set_expression"];
    let event = Event::build(&full_path, Some(expression));
    let result = model.perform_event(&event, &full_path[1..]).unwrap();

    // check if changed correctly
    assert_eq!(
        model.get_uninterpreted_fn(&f).unwrap().get_fn_expression(),
        expression,
    );

    check_reverse(
        model,
        model_orig,
        result,
        &["uninterpreted_fn", f.as_str(), "set_expression"],
    );
}

#[test]
fn test_refresh() {
    let mut model = ModelState::new();
    let layout_id = ModelState::get_default_layout_id().to_string();
    let var_a = model.generate_var_id("a");
    let function_f = model.generate_uninterpreted_fn_id("f");
    let expression = "a | a";

    model.add_var(var_a.clone(), "a_name").unwrap();
    model.add_regulation_by_str("a -> a").unwrap();
    model
        .add_new_uninterpreted_fn(function_f.clone(), "f_name", 2)
        .unwrap();
    model.set_update_fn(&var_a, expression).unwrap();

    // test variable getter
    let event = model
        .refresh(
            &["model".to_string(), "get_variables".to_string()],
            &["get_variables"],
        )
        .unwrap();
    let var_list: Vec<VariableData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(var_list.len(), 1);
    assert_eq!(var_list[0].name, "a_name".to_string());
    assert_eq!(var_list[0].id, var_a.to_string());
    assert_eq!(var_list[0].update_fn, expression);

    // test regulation getter
    let event = model
        .refresh(
            &["model".to_string(), "get_regulations".to_string()],
            &["get_regulations"],
        )
        .unwrap();
    let reg_list: Vec<RegulationData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(reg_list.len(), 1);
    assert_eq!(reg_list[0].target, var_a.to_string());
    assert_eq!(reg_list[0].regulator, var_a.to_string());

    // test layout getter
    let event = model
        .refresh(
            &["model".to_string(), "get_layouts".to_string()],
            &["get_layouts"],
        )
        .unwrap();
    let layout_list: Vec<LayoutData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(layout_list.len(), 1);
    assert_eq!(layout_list[0].id, layout_id.clone());

    // test layout node getter
    let event = model
        .refresh(
            &[
                "model".to_string(),
                "get_layout_nodes".to_string(),
                layout_id.clone(),
            ],
            &["get_layout_nodes", &layout_id],
        )
        .unwrap();
    let node_list: Vec<LayoutNodeData> = serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(node_list.len(), 1);
    assert_eq!(node_list[0].variable, var_a.to_string());

    // test uninterpreted fn getter
    let event = model
        .refresh(
            &["model".to_string(), "get_uninterpreted_fns".to_string()],
            &["get_uninterpreted_fns", function_f.as_str()],
        )
        .unwrap();
    let uninterpreted_fn_list: Vec<UninterpretedFnData> =
        serde_json::from_str(&event.payload.unwrap()).unwrap();
    assert_eq!(uninterpreted_fn_list.len(), 1);
    assert_eq!(uninterpreted_fn_list[0].id, function_f.to_string());
}
