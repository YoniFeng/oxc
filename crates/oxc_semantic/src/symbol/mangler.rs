use std::cell::RefCell;

use oxc_span::Atom;

use super::Symbol;

/// # Name Mangler / Symbol Minification
///
/// See:
///   * [esbuild](https://github.com/evanw/esbuild/blob/main/docs/architecture.md#symbol-minification)
///
/// This algorithm is targeted for better gzip compression.
///
/// Visually, a slot is the index position for binding identifiers:
///
/// ```javascript
/// function x(slot0, slot1) {
///     function y(slot2, slot3) {
///         slot0 = 1;
///     }
/// }
/// function z(slot0, slot1, slot3, slot4) {
///     slot0 = 1;
/// }
/// ```
///
/// Occurrences of slots and their corresponding newly assigned short identifiers (mangled names) are:
/// - slot0: 4 - a
/// - slot1: 2 - b
/// - slot3: 2 - c
/// - slot2: 1 - d
/// - slot4: 1 - e
///
/// After swapping out the mangled names, the functions become:
///
/// ```javascript
/// function x(a, b) {
///     function y(d, c) {
///         a = 1;
///     }
/// }
/// function z(a, b, c, e) {
///     a = 1;
/// }
/// ```
#[derive(Debug, Default)]
pub struct Mangler {
    /// The current slot used by semantic builder
    current_slot: Slot,

    /// The maximum slot of all scopes.
    max_slot: Slot,

    /// Mangled names, indexed by slot, length by `max_slot`.
    mangled_names: RefCell<Vec<Atom>>,
}

/// A slot is the occurrence index of a binding identifier inside a scope.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Slot(usize);

impl Slot {
    #[inline]
    pub fn index(self) -> usize {
        self.0 - 1
    }

    #[inline]
    pub fn is_some(self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

impl Mangler {
    pub fn next_slot(&mut self) -> Slot {
        let slot = self.current_slot;
        self.current_slot.increment();
        if self.current_slot > self.max_slot {
            self.max_slot = self.current_slot;
        }
        slot
    }

    pub fn reset_slot(&mut self) {
        self.current_slot = Slot(1);
    }

    pub fn mangled_name(&self, slot: Slot) -> Atom {
        self.mangled_names.borrow()[slot.index()].clone()
    }

    pub fn compute_slot_frequency(&self, symbols: &[Symbol]) {
        // (index, frequency)
        let mut frequencies = vec![(0usize, 0usize); self.max_slot.index()];
        for symbol in symbols {
            if symbol.slot.is_some() {
                let index = symbol.slot.index();
                frequencies[index].0 = index;
                frequencies[index].1 += symbol.references.len() + 1;
            }
        }
        frequencies.sort_by_key(|x| std::cmp::Reverse(x.1));
        let mut mangled_names = vec![Atom::from(""); self.max_slot.index()];
        let mut i = 0;
        for freq in &frequencies {
            let keyword = loop {
                let keyword = Atom::base54(i);
                i += 1;
                if !Self::is_keyword(keyword.as_str()) {
                    break keyword;
                }
            };
            mangled_names[freq.0] = keyword;
        }
        *self.mangled_names.borrow_mut() = mangled_names;
    }

    #[rustfmt::skip]
    fn is_keyword(s: &str) -> bool {
        let len = s.len();
        if len == 1 {
            return false;
        }
        matches!(s, "as" | "do" | "if" | "in" | "is" | "of" | "any" | "for" | "get"
                | "let" | "new" | "out" | "set" | "try" | "var" | "case" | "else"
                | "enum" | "from" | "meta" | "null" | "this" | "true" | "type"
                | "void" | "with")
    }
}
