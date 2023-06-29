use crate::debug;
use crate::gui::events::GuiEvent;
use std::collections::VecDeque;

pub const DEFAULT_EVENT_LIMIT: usize = 1 << 16; // ~64k
pub const DEFAULT_PAYLOAD_LIMIT: usize = 1 << 28; // 256MB

/// The items on the undo/redo stack are pairs of events: one performs the actual action
/// while the other reverses said action.
///
/// Note that some actions cannot be reversed or can be performed only once. These cannot be
/// a part of the undo/redo stack, but there are other ways for triggering those.
#[derive(Clone, Eq, PartialEq)]
pub struct UndoStackEntry {
    perform_action: GuiEvent,
    reverse_action: GuiEvent,
}

impl UndoStackEntry {
    /// The sum of payload sizes for the underlying UI actions.
    pub fn payload_size(&self) -> usize {
        self.perform_action.payload_size() + self.reverse_action.payload_size()
    }
}

/// The stack that keeps track of all the events that
#[derive(Clone, Eq, PartialEq)]
pub struct UndoStack {
    /// The number of events this `UndoStack` is allowed to track.
    /// Beyond this limit, the stack will start dropping the oldest events.
    event_limit: usize,

    /// The number of bytes of payload that this `UndoStack` is allowed to store.
    /// Beyond this limit, the stack will start dropping the oldest events.
    payload_limit: usize,

    /// The approximate size of all payloads stored on the `undo_stack`.
    ///
    /// Note that the `redo_stack` does not count towards this value, because
    /// entries only move there from the `undo_stack` (i.e. after the payload size check)
    /// and when we push to the `undo_stack`, the `redo_stack` is erased. Hence,
    /// assuming all invariants hold, `undo_stack + redo_stack < limit`, even if we
    /// just track the `undo_stack`.
    current_payload_size: usize,

    undo_stack: VecDeque<UndoStackEntry>,
    redo_stack: VecDeque<UndoStackEntry>,
}

impl UndoStack {
    pub fn new(event_limit: usize, payload_limit: usize) -> UndoStack {
        UndoStack {
            event_limit,
            payload_limit,
            current_payload_size: 0,
            undo_stack: VecDeque::with_capacity(event_limit),
            redo_stack: VecDeque::with_capacity(event_limit),
        }
    }

    /// The number of events that can be un-done.
    pub fn undo_len(&self) -> usize {
        self.undo_stack.len()
    }

    /// The number of events that can be re-done.
    pub fn redo_len(&self) -> usize {
        self.redo_stack.len()
    }

    /// Notify the undo stack that a new action will be performed. This creates a new stack
    /// entry for this action. Furthermore, it erases any available "redo" actions.
    ///
    /// Returns `true` if the events were successfully saved, or `false` if an error occurred,
    /// e.g. due to excessive payload size.
    #[must_use]
    pub fn do_action(&mut self, perform: GuiEvent, reverse: GuiEvent) -> bool {
        // Items from `redo_stack` are no longer relevant since the
        self.redo_stack.clear();

        // Drop events that are over limit (first event limit, then payload limit).
        while self.undo_stack.len() >= self.event_limit {
            let Some(event) = self.drop_undo_event() else {
                break;  // The stack is empty.
            };
            debug!(
                "Event count exceeded. Dropping event: `{}`.",
                event.perform_action.action()
            );
        }
        let additional_payload = perform.payload_size() + reverse.payload_size();
        while self.current_payload_size + additional_payload >= self.payload_limit {
            let Some(event) = self.drop_undo_event() else {
                break;  // The stack is empty.
            };
            debug!(
                "Payload size exceeded. Dropping event: `{}`.",
                event.perform_action.action()
            );
        }

        // Now we can actually push the entry into the stack. But only if some limit is not
        // preventing us from doing so. This generally should not happen, but it can occur
        // either if `event_limit == 0` or if `additional_payload` is very high and thus we
        // cannot store the event at all.

        if self.undo_stack.len() >= self.event_limit {
            assert!(self.undo_stack.is_empty());
            debug!("Cannot save new undo item. Event limit is likely zero.");
            return false;
        }

        if self.current_payload_size + additional_payload >= self.payload_limit {
            assert_eq!(self.current_payload_size, 0);
            debug!(
                "Cannot save new undo item. Event payload too large: {} > {}.",
                additional_payload, self.payload_limit
            );
            return false;
        }

        self.undo_stack.push_back(UndoStackEntry {
            perform_action: perform,
            reverse_action: reverse,
        });
        self.current_payload_size += additional_payload;

        true
    }

    /// Try to undo the current top of the undo stack. This action can be later re-done using
    /// `Self::redo_action`. Returns `None` if there is no action to undo, or the "reverse"
    /// `GuiEvent` originally supplied to `Self::do_action`.
    #[must_use]
    pub fn undo_action(&mut self) -> Option<GuiEvent> {
        let Some(entry) = self.undo_stack.pop_back() else {
            return None;
        };

        let result = Some(entry.reverse_action.clone());
        self.current_payload_size -= entry.payload_size();
        self.redo_stack.push_back(entry);
        result
    }

    /// Try to redo the current top of the redo stack. This action can be later un-done using
    /// `Self::undo_action`. Returns `None` if there is no action to redo, or the "perform"
    /// `GuiEvent` originally supplied to `Self::do_action`.
    #[must_use]
    pub fn redo_action(&mut self) -> Option<GuiEvent> {
        let Some(entry) = self.redo_stack.pop_back() else {
            return None;
        };

        let result = Some(entry.perform_action.clone());
        self.current_payload_size += entry.payload_size();
        self.undo_stack.push_back(entry);
        result
    }

    /// Internal function to drop an entry from the `undo_stack`.
    fn drop_undo_event(&mut self) -> Option<UndoStackEntry> {
        let entry = self.undo_stack.pop_front()?;
        self.current_payload_size -= entry.payload_size();
        assert!(self.current_payload_size < self.payload_limit);
        Some(entry)
    }
}

impl Default for UndoStack {
    fn default() -> Self {
        UndoStack::new(DEFAULT_EVENT_LIMIT, DEFAULT_PAYLOAD_LIMIT)
    }
}

#[cfg(test)]
mod tests {
    use crate::gui::events::GuiEvent;
    use crate::gui::undo_stack::UndoStack;

    #[test]
    pub fn test_normal_behaviour() {
        let mut stack = UndoStack::default();
        let e1 = GuiEvent::with_action(&[], "action 1");
        let e2 = GuiEvent::with_action(&[], "action 2");

        // We can do a bunch of events and undo/redo them.
        assert!(stack.do_action(e1.clone(), e2.clone()));
        assert!(stack.do_action(e2.clone(), e1.clone()));
        assert_eq!(2, stack.undo_len());
        assert_eq!(Some(e1.clone()), stack.undo_action());
        assert_eq!(1, stack.undo_len());
        assert_eq!(1, stack.redo_len());
        assert_eq!(Some(e2.clone()), stack.undo_action());
        assert_eq!(None, stack.undo_action());
        assert_eq!(Some(e1.clone()), stack.redo_action());
        assert_eq!(Some(e2.clone()), stack.redo_action());
        assert_eq!(None, stack.redo_action());
        assert_eq!(2, stack.undo_len());

        // If we start doing something else, the redo stack is lost.
        assert_eq!(Some(e1.clone()), stack.undo_action());
        assert_eq!(1, stack.redo_len());
        assert!(stack.do_action(e1.clone(), e2.clone()));
        assert_eq!(0, stack.redo_len());
        assert_eq!(2, stack.undo_len());
        assert_eq!(Some(e2.clone()), stack.undo_action());
    }

    #[test]
    pub fn test_basic_limits() {
        let e1 = GuiEvent::with_action(&[], "action 1");
        let e2 = GuiEvent::with_action(&[], "action 2");
        let e3 =
            GuiEvent::with_action_and_payload(&["path".to_string()], "action 3", "some payload");

        let mut stack = UndoStack::new(4, 2 * e3.payload_size() + 1);

        // Test that the even limit is respected. We should be able to fit 4 events.
        assert!(stack.do_action(e2.clone(), e1.clone()));
        assert!(stack.do_action(e1.clone(), e2.clone()));
        assert!(stack.do_action(e1.clone(), e2.clone()));
        assert_eq!(3, stack.undo_len());
        assert!(stack.do_action(e1.clone(), e2.clone()));
        assert_eq!(4, stack.undo_len());
        // Here, an event should be dropped, but we can still undo the events.
        assert!(stack.do_action(e1.clone(), e2.clone()));
        assert_eq!(4, stack.undo_len());
        assert_eq!(Some(e2.clone()), stack.undo_action());
        assert_eq!(Some(e2.clone()), stack.undo_action());
        assert_eq!(Some(e2.clone()), stack.undo_action());
        assert_eq!(Some(e2.clone()), stack.undo_action());
        assert_eq!(None, stack.undo_action());

        // Test that the payload limit is respected. We should be able to fit two events
        // including payload.
        assert!(stack.do_action(e2.clone(), e1.clone()));
        assert!(stack.do_action(e3.clone(), e1.clone()));
        assert!(stack.do_action(e2.clone(), e3.clone()));
        assert_eq!(3, stack.undo_len());
        // This should drop the first two events.
        assert!(stack.do_action(e2.clone(), e3.clone()));
        assert_eq!(2, stack.undo_len());
        // This should just drop everything except for the last event.
        assert!(stack.do_action(e3.clone(), e3.clone()));
        assert_eq!(1, stack.undo_len());
        assert_eq!(Some(e3.clone()), stack.undo_action());
        assert_eq!(Some(e3.clone()), stack.redo_action());
    }

    #[test]
    pub fn test_extreme_limits() {
        let e1 = GuiEvent::with_action(&[], "action 1");
        let e2 = GuiEvent::with_action(&[], "action 2");
        let e3 =
            GuiEvent::with_action_and_payload(&["path".to_string()], "action 3", "some payload");

        let mut stack = UndoStack::new(0, 1024);
        // Cannot perform action because the stack size is zero.
        assert!(!stack.do_action(e1.clone(), e2.clone()));

        let mut stack = UndoStack::new(8, 8);
        // Can push events without payload, but events with payload are not possible.
        assert!(stack.do_action(e1.clone(), e2.clone()));
        assert!(!stack.do_action(e1.clone(), e3.clone()));
        assert_eq!(0, stack.undo_len());
    }
}
