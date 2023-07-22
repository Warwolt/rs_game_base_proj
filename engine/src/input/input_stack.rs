#[derive(Debug)]
pub struct InputStack<T> {
    items: Vec<T>,
}

impl<T: PartialEq> InputStack<T> {
    pub fn new() -> Self {
        InputStack { items: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn remove(&mut self, item: &T) {
        if let Some(index) = self.items.iter().position(|i| i == item) {
            self.items.remove(index);
        }
    }

    pub fn top(&self) -> Option<&T> {
        self.items.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initially_empty() {
        let input_stack = InputStack::<u32>::new();
        assert_eq!(input_stack.top(), None);
    }

    #[test]
    fn item_is_on_top_after_being_pushed() {
        let mut input_stack = InputStack::<u32>::new();

        input_stack.push(32);

        assert_eq!(input_stack.top(), Some(&32));
    }

    #[test]
    fn item_in_middle_can_be_removed() {
        let mut input_stack = InputStack::<u32>::new();

        input_stack.push(1);
        input_stack.push(2);
        input_stack.remove(&1);

        assert_eq!(input_stack.top(), Some(&2));
    }
}
