mod heap;
mod pointer;
mod ui;

pub use heap::{Heap, HeapData, HeapValue};
pub use pointer::{Pointer, PointerKind};
pub use ui::{DynamicProperty, UIProperty};

/// Memory contains all the data created by our runtime.
///
/// When say a boolean is created in ftd world, we add an entry in the `.boolean` here, and return
/// the "pointer" to this to wasm world as `externref` type. Similarly we have `.i32`, and `.f32`.
///
/// Currently we store all integers (`i8`, `u8` etc) as `i32` and all floats as `f32`. These are
/// the types in wasm and we are designed to be used with wasm only.
///
/// For vectors and structs, we use a memory sub-optimal solution of storing each data as a vector,
/// so a vector containing two booleans will be a vector containing two pointers pointing to each
/// boolean, instead of storing the booleans themselves.
///
/// we store enums in `.or_type`. The `u8` is for storing the variant of the enum that this
/// value represents. The data for the variant is stored in the Vec.
///
/// We maintain stack of function calls in a `.stack`. We do not store any data on stack, the
/// purpose of stack is to assist in garbage collection. When a value is created it's pointer is
/// stored on the top frame of the stack. When we attach any value to dom using `.attach_to_dom()`
/// we remove the pointer and all the descendants of the pointer from the frame they were created
/// in. This was at the end of the frame, whatever is left is safe to de-allocate.
///
/// The real magic happens when `.attach_to_dom()` is called on any pointer. We call this the
/// "pointer getting attached to the UI". Any pointer that is not attached to UI gets de-allocated
/// at first opportunity.
///
/// When a pointer is created, we also create a `Vec<Attachment>`, and store it next to it. So if
/// a boolean is created we create a store both the boolean and `Vec<Attachment>` for that boolean
/// in the `.boolean`. We have a type `PointerData<T>` which keeps track of the value and the
/// attachments.
///
/// When `.attach_to_dom()` is called, we find all the dependencies.
///
/// if we have:
/// -- ftd.text: hello
///
/// a string containing hello will be created, and then passed to Rust as text properties, and
/// original wasm value would get dropped.
#[derive(Debug, Default)]
pub struct Memory {
    /// when a function starts in wasm side, a new `Frame` is created and added here. Each new
    /// pointer we create, we add it to the `Frame`. When a new pointer is created, it is
    /// considered "owned" by the `Frame`. Once we attach to dom node using `Memory.attach_to_dom()`,
    /// we remove the link to pointer from the frame. This way at the end of the frame we see if
    /// anything is still attached to the frame, and which means that pointer is not attached to
    /// anything else, we clear it up cleanly.
    stack: Vec<Frame>,

    boolean: Heap<bool>,
    i32: Heap<i32>,
    f32: Heap<f32>,
    /// `.vec` can store both `vec`s, `tuple`s, and `struct`s using these. For struct the fields
    /// are stored in the order they are defined.
    pub vec: Heap<Vec<Pointer>>,
    or_type: Heap<(u8, Vec<Pointer>)>,

    closures: slotmap::SlotMap<fastn_runtime::ClosureKey, Closure>,
    // if we have:
    // -- ftd.text: hello
    //
    // a string containing hello will be created, and then passed to Rust as text properties, and
    // original wasm value would get dropped.
}

#[derive(Debug)]
pub struct Closure {
    /// functions are defined in wasm, and this is the index in the function table.
    pub function: i32,
    /// function_data is the pointer to a vector that contains all the variables "captured" by this
    /// closure.
    pub captured_variables: Pointer,

    // in future we can this optimisation: Saves us from creating vectors unless needed. Most
    // closures have two pointers (if most had three we can create a v3).

    // pub v1: Pointer,
    // pub v2: Option<Pointer>,
    // pub rest: Option<Vec<Pointer>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Attachment {
    /// this is the dom element we are directly or indirectly connected with
    element: fastn_runtime::NodeKey,
    /// who told we us about this element
    source: Pointer,
}

#[derive(Debug, Default)]
pub struct Frame {
    pointers: Vec<Pointer>,
}

impl Memory {
    #[cfg(test)]
    #[track_caller]
    pub(crate) fn assert_empty(&self) {
        if !self.stack.is_empty() {
            panic!("stack is not empty");
        }
        if !self.boolean.is_empty() {
            panic!("boolean is not empty");
        }
        if !self.i32.is_empty() {
            panic!("i32 is not empty");
        }
        if !self.f32.is_empty() {
            panic!("f32 is not empty");
        }
        if !self.vec.is_empty() {
            panic!("vec is not empty");
        }
        if !self.or_type.is_empty() {
            panic!("or_type is not empty");
        }
        if !self.closures.is_empty() {
            panic!("closures is not empty");
        }
    }

    pub fn get_colors(&self, color_pointer: fastn_runtime::PointerKey) -> (i32, i32, i32, f32) {
        let vec_value = self
            .vec
            .get(color_pointer)
            .expect("Expected color vec")
            .value
            .value();
        let r_pointer = vec_value.get(0).expect("Expected r pointer");
        let r_value = self
            .i32
            .get(r_pointer.key)
            .expect("Expected r value")
            .value
            .value();

        let g_pointer = vec_value.get(1).expect("Expected g pointer");
        let g_value = self
            .i32
            .get(g_pointer.key)
            .expect("Expected g value")
            .value
            .value();

        let b_pointer = vec_value.get(2).expect("Expected b pointer");
        let b_value = self
            .i32
            .get(b_pointer.key)
            .expect("Expected b value")
            .value
            .value();

        let a_pointer = vec_value.get(3).expect("Expected a pointer");
        let a_value = self
            .f32
            .get(a_pointer.key)
            .expect("Expected a value")
            .value
            .value();

        (*r_value, *g_value, *b_value, *a_value)
    }

    pub fn detach_dom(&mut self, _dom: fastn_runtime::NodeKey) {
        // for pointer in self.ui_deps.remove(&dom).unwrap_or_default() {
        //     self.drop_pointer(&pointer);
        // }
    }

    pub fn attach_to_dom(&mut self, _dom: fastn_runtime::NodeKey, _ptr: Pointer) {
        // add a new dependency to ptr, and recursively add it to all its dependencies
        todo!()
    }

    fn get_pointer_dep_children(&self, _pointer: &Pointer) -> Option<Vec<Pointer>> {
        // match &pointer.kind {
        //     PointerKind::Boolean => self.boolean.get(pointer.key).map(|v| v.value.clone()),
        //     PointerKind::Integer => self.boolean.get(pointer.key).map(|v| v.1.clone()),
        //     PointerKind::Record => self
        //         .vec
        //         .get(pointer.key)
        //         .map(|v| [v.0.clone(), v.1.clone()].concat().into_iter().collect()),
        //     PointerKind::OrType => self.or_type.get(pointer.key).map(|v| v.1.clone()),
        //     PointerKind::Decimal => self.f32.get(pointer.key).map(|v| v.1.clone()),
        // }
        todo!()
    }

    fn add_dep_child(&mut self, _pointer: &Pointer, _child: Pointer) {
        // if let Some(dep_children) = match &pointer.kind {
        //     PointerKind::Boolean => self.boolean.get_mut(pointer.key).map(|v| &mut v.1),
        //     PointerKind::Integer => self.boolean.get_mut(pointer.key).map(|v| &mut v.1),
        //     PointerKind::Record => self.vec.get_mut(pointer.key).map(|v| &mut v.1),
        //     PointerKind::OrType => self.or_type.get_mut(pointer.key).map(|v| &mut v.1),
        //     PointerKind::Decimal => self.f32.get_mut(pointer.key).map(|v| &mut v.1),
        // } {
        //     dep_children.push(child);
        // }
        todo!()
    }

    pub fn attach(&mut self, _parent: Pointer, _child: Pointer) {
        // let parent_attachments = if let Some(attachment) = self.attachment.get(&parent) {
        //     attachment.clone()
        // } else {
        //     return;
        // };
        // let mut child_attachments = self.attachment.entry(child.clone()).or_default().clone();
        // for parent_attachment in parent_attachments {
        //     // if parent has not already given the attachment to the child, add it
        //     let attachment = Attachment {
        //         element: parent_attachment.element,
        //         source: parent.clone(),
        //     };
        //     let is_attached = child_attachments.insert(attachment);
        //     if is_attached {
        //         let dep_children = self.get_pointer_dep_children(&child).unwrap();
        //         for dep in dep_children {
        //             self.attach(child.clone(), dep)
        //         }
        //     }
        // }
        //
        // *self.attachment.get_mut(&child).unwrap() = child_attachments;
        // self.add_dep_child(&parent, child.clone());
        // // TODO: pass all attachments from parent to child
        // self.drop_from_frame(&child);
    }

    fn insert_in_frame(&mut self, pointer: fastn_runtime::PointerKey, kind: PointerKind) {
        // using .unwrap() so we crash on a bug instead of silently ignoring it
        self.stack
            .last_mut()
            .unwrap()
            .pointers
            .push(Pointer { key: pointer, kind });
    }

    pub fn create_frame(&mut self) {
        self.stack.push(Frame::default());
    }

    fn drop_from_frame(&mut self, _pointer: &Pointer) {
        todo!()
    }

    fn drop_pointer(&mut self, pointer: &Pointer) -> bool {
        let (dependents, ui_properties) = match pointer.kind {
            PointerKind::Boolean => {
                let b = self.boolean.get(pointer.key).unwrap();
                (&b.dependents, &b.ui_properties)
            }
            PointerKind::Integer => {
                let b = self.i32.get(pointer.key).unwrap();
                (&b.dependents, &b.ui_properties)
            }
            PointerKind::Record | PointerKind::List => {
                let b = self.vec.get(pointer.key).unwrap();
                (&b.dependents, &b.ui_properties)
            }
            PointerKind::OrType => {
                let b = self.or_type.get(pointer.key).unwrap();
                (&b.dependents, &b.ui_properties)
            }
            PointerKind::Decimal => {
                let b = self.f32.get(pointer.key).unwrap();
                (&b.dependents, &b.ui_properties)
            }
        };

        if !ui_properties.is_empty() {
            return false;
        }

        let mut drop = true;
        for d in dependents.clone() {
            if !self.drop_pointer(&d) {
                drop = false;
                break;
            }
        }

        if drop {
            self.delete_pointer(pointer);
        }

        drop
    }

    fn delete_pointer(&mut self, pointer: &Pointer) {
        match pointer.kind {
            PointerKind::Boolean => {
                self.boolean.remove(pointer.key);
            }
            PointerKind::Integer => {
                self.i32.remove(pointer.key);
            }
            PointerKind::Record | PointerKind::List => {
                self.vec.remove(pointer.key);
            }
            PointerKind::OrType => {
                self.or_type.remove(pointer.key);
            }
            PointerKind::Decimal => {
                self.f32.remove(pointer.key);
            }
        };
    }

    pub fn end_frame(&mut self) {
        // using .unwrap() so we crash on a bug instead of silently ignoring it
        for pointer in self.stack.pop().unwrap().pointers.iter() {
            self.drop_pointer(pointer);
        }
    }

    pub fn return_frame(&mut self, _k: fastn_runtime::PointerKey) -> fastn_runtime::PointerKey {
        todo!()
    }

    pub(crate) fn create_closure(&mut self, closure: Closure) -> fastn_runtime::ClosureKey {
        self.closures.insert(closure)
    }

    pub fn create_boolean(&mut self, value: bool) -> fastn_runtime::PointerKey {
        let pointer = self.boolean.insert(HeapValue::new(value).into_heap_data());
        self.insert_in_frame(pointer, PointerKind::Boolean);
        pointer
    }

    pub fn is_pointer_valid(&self, ptr: fastn_runtime::Pointer) -> bool {
        match ptr.kind {
            fastn_runtime::PointerKind::Boolean => self.boolean.contains_key(ptr.key),
            _ => todo!()
        }
    }
    pub fn get_boolean(&mut self, ptr: fastn_runtime::PointerKey) -> bool {
        *self.boolean[ptr].value.value()
    }

    pub fn set_boolean(&mut self, ptr: fastn_runtime::PointerKey, value: bool) {
        self.boolean[ptr].value.set_value(value)
    }

    pub fn create_i32(&mut self, value: i32) -> fastn_runtime::PointerKey {
        let pointer = self.i32.insert(HeapValue::new(value).into_heap_data());
        self.insert_in_frame(pointer, PointerKind::Integer);
        pointer
    }

    pub fn get_i32(&mut self, ptr: fastn_runtime::PointerKey) -> i32 {
        *self.i32[ptr].value.value()
    }

    pub fn set_i32(&mut self, ptr: fastn_runtime::PointerKey, value: i32) {
        self.i32[ptr].value.set_value(value)
    }

    pub fn create_i32_func(
        &mut self,
        cached_value: i32,
        closure: Closure,
    ) -> fastn_runtime::PointerKey {
        let closure_key = self.create_closure(closure);
        let pointer = self
            .i32
            .insert(HeapValue::new_with_formula(cached_value, closure_key).into_heap_data());
        self.insert_in_frame(pointer, PointerKind::Integer);
        pointer
    }

    pub fn get_func_arg_i32(&self, ptr: fastn_runtime::PointerKey, idx: i32) -> i32 {
        let ptr = self
            .vec
            .get(ptr)
            .unwrap()
            .value
            .value()
            .get(idx as usize)
            .unwrap();
        *self.i32.get(ptr.key).unwrap().value.value()
    }

    pub fn array_i32_2(
        &mut self,
        ptr1: fastn_runtime::PointerKey,
        ptr2: fastn_runtime::PointerKey,
    ) -> fastn_runtime::PointerKey {
        let vec = self.vec.insert(
            HeapValue::new(vec![
                Pointer {
                    key: ptr1,
                    kind: PointerKind::Integer,
                },
                Pointer {
                    key: ptr2,
                    kind: PointerKind::Integer,
                },
            ])
            .into_heap_data(),
        );
        self.add_dependent(ptr1.into_integer_pointer(), vec.into_list_pointer());
        self.add_dependent(ptr2.into_integer_pointer(), vec.into_list_pointer());
        vec
    }

    pub fn add_dependent(&mut self, target: Pointer, dependent: Pointer) {
        let dependents = match target.kind {
            PointerKind::Integer => &mut self.i32.get_mut(target.key).unwrap().dependents,
            PointerKind::Boolean => &mut self.boolean.get_mut(target.key).unwrap().dependents,
            PointerKind::Decimal => &mut self.f32.get_mut(target.key).unwrap().dependents,
            PointerKind::List | PointerKind::Record | PointerKind::OrType => {
                &mut self.vec.get_mut(target.key).unwrap().dependents
            }
        };

        dependents.push(dependent);
    }

    pub fn add_ui_dependent(&mut self, target: Pointer, dependent: DynamicProperty) {
        let dependents = match target.kind {
            PointerKind::Integer => &mut self.i32.get_mut(target.key).unwrap().ui_properties,
            PointerKind::Boolean => &mut self.boolean.get_mut(target.key).unwrap().ui_properties,
            PointerKind::Decimal => &mut self.f32.get_mut(target.key).unwrap().ui_properties,
            PointerKind::List | PointerKind::Record | PointerKind::OrType => {
                &mut self.vec.get_mut(target.key).unwrap().ui_properties
            }
        };

        dependents.push(dependent);
    }

    pub fn create_rgba(&mut self, r: i32, g: i32, b: i32, a: f32) -> fastn_runtime::PointerKey {
        let r_pointer = self.i32.insert(HeapValue::new(r).into_heap_data());
        let g_pointer = self.i32.insert(HeapValue::new(g).into_heap_data());
        let b_pointer = self.i32.insert(HeapValue::new(b).into_heap_data());
        let a_pointer = self.f32.insert(HeapValue::new(a).into_heap_data());

        let vec = self.vec.insert(
            HeapValue::new(vec![
                Pointer {
                    key: r_pointer,
                    kind: PointerKind::Integer,
                },
                Pointer {
                    key: g_pointer,
                    kind: PointerKind::Integer,
                },
                Pointer {
                    key: b_pointer,
                    kind: PointerKind::Integer,
                },
                Pointer {
                    key: a_pointer,
                    kind: PointerKind::Decimal,
                },
            ])
            .into_heap_data(),
        );

        self.add_dependent(r_pointer.into_integer_pointer(), vec.into_record_pointer());
        self.add_dependent(g_pointer.into_integer_pointer(), vec.into_record_pointer());
        self.add_dependent(b_pointer.into_integer_pointer(), vec.into_record_pointer());
        self.add_dependent(a_pointer.into_integer_pointer(), vec.into_record_pointer());

        self.insert_in_frame(vec, PointerKind::Record);
        vec
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn create_get_and_set() {
        let mut m = super::Memory::default();
        println!("{:#?}", m);
        m.assert_empty();
        m.create_frame();

        let p = m.create_boolean(true);
        assert!(m.get_boolean(p));

        m.set_boolean(p, false);
        assert!(!m.get_boolean(p));

        let p = m.create_boolean(false);
        assert!(!m.get_boolean(p));

        let p = m.create_i32(20);
        assert_eq!(m.get_i32(p), 20);

        m.set_i32(p, 30);
        assert_eq!(m.get_i32(p), 30);

        println!("{:#?}", m);
        m.end_frame();
        m.assert_empty();
        println!("{:#?}", m);
    }

    #[test]
    fn stack() {
        let mut m = super::Memory::default();
        println!("{:#?}", m);
        m.assert_empty();

        {
            m.create_frame();

            let p = m.create_boolean(true);
            assert!(m.get_boolean(p));

            {
                m.create_frame();
                assert!(m.get_boolean(p));

                let p2 = m.create_boolean(false);
                assert!(!m.get_boolean(p2));

                m.end_frame();
                assert!(m.is_pointer_valid(p.into_boolean_pointer()));
                assert!(!m.is_pointer_valid(p2.into_boolean_pointer()));
            }

            assert!(m.get_boolean(p));
            m.end_frame();
            assert!(!m.is_pointer_valid(p.into_boolean_pointer()));
        }

        m.assert_empty();
    }
}

// -- record x:
// y list y:
//
// -- record y:
// string z:
//
// -- x $x:
// -- x.y:
// z: hello

// -- foo: $x.y
// -- ftd.text: $x.y.z

// -- ftd.text: yo
// $on-click$: $x = new_x(x, "bye")
// $on-click$: $x.y = new_y("bye")

// -- l: $o
// $loop$: $x.y

// x.y.z = "hello"
// x.y.z changed

// (attach_dom (create_l) $x [0, 0])

// (attach_dom (create_l) $x [0, 0])

// x.y.insert_at(0, new_y)

// (attach_dom (create_text) $x [0, 0])

// -- foo:
// person: $person

// -- foo:
// $person: $person

// -- show-student: $student
// $loop$: $students as $student
// rank: calculate_rank($students, idx)

// -- ftd.text:
// $on-click$: $x = new_x(x, "bye")
// $on-click$: $x.y = new_y("bye")
//
// x new_x(v):
// string v:
//
// {
//    y: {
//        z: v
//    }
// }
