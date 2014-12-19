use std::mem;
use std::mem::{transmute, size_of, replace};
use std::default::Default;

pub trait SimpleSerialize<R: SimpleSerializeVec<Self>> { }

pub trait SimpleSerializeVec<T> : Default {
    fn push(&mut self, T);
    fn pop(&mut self) -> Option<T>;

    fn encode(&mut self, &mut Vec<Vec<u8>>);
    fn decode(&mut self, &mut Vec<Vec<u8>>);
}

impl<T:Copy> SimpleSerialize<Vec<T>> for T { }

impl<T: SimpleSerialize<R>, R: SimpleSerializeVec<T>> SimpleSerialize<(Vec<uint>, R, Vec<Vec<T>>)> for Vec<T> { }

impl<T:Copy> SimpleSerializeVec<T> for Vec<T> {
    #[inline(always)]
    fn push(&mut self, data: T) { self.push(data); }

    #[inline(always)]
    fn pop(&mut self) -> Option<T> { self.pop() }

    fn encode(&mut self, buffers: &mut Vec<Vec<u8>>) {
        buffers.push(unsafe { to_bytes_vec(replace(self, Vec::new())) });
    }

    fn decode(&mut self, buffers: &mut Vec<Vec<u8>>)
    {
        if self.len() > 0 { panic!("calling decode from a non-empty SimpleSerializeVec"); }
        *self = unsafe { to_typed_vec(buffers.pop().unwrap()) };
    }
}

impl<T, R1: SimpleSerializeVec<uint>, R2: SimpleSerializeVec<T>> SimpleSerializeVec<Vec<T>> for (R1, R2, Vec<Vec<T>>)
{
    #[inline(always)]
    fn push(&mut self, mut vector: Vec<T>)
    {
        self.0.push(vector.len());
        while let Some(record) = vector.pop() { self.1.push(record); }
        self.2.push(vector);
    }

    #[inline(always)]
    fn pop(&mut self) -> Option<Vec<T>>
    {
        if let Some(count) = self.0.pop()
        {
            let mut vector = self.2.pop().unwrap_or(Vec::new());
            for _ in range(0, count) { vector.push(self.1.pop().unwrap()); }
            Some(vector)
        }
        else { None }
    }

    fn encode(&mut self, buffers: &mut Vec<Vec<u8>>)
    {
        self.0.encode(buffers);
        self.1.encode(buffers);
    }

    fn decode(&mut self, buffers: &mut Vec<Vec<u8>>)
    {
        self.1.decode(buffers);
        self.0.decode(buffers);
    }
}

unsafe fn to_typed_vec<T>(mut vector: Vec<u8>) -> Vec<T>
{
    let rawbyt: *mut u8 = vector.as_mut_ptr();

    let length = vector.len() / size_of::<T>();
    let rawptr: *mut T = transmute(rawbyt);
    mem::forget(vector);

    Vec::from_raw_parts(rawptr, length, length)
}

unsafe fn to_bytes_vec<T>(mut vector: Vec<T>) -> Vec<u8>
{
    let rawbyt: *mut T = vector.as_mut_ptr();

    let length = vector.len() * size_of::<T>();
    let rawptr: *mut u8 = transmute(rawbyt);
    mem::forget(vector);

    Vec::from_raw_parts(rawptr, length, length)
}



// tests!
#[test]
fn test_uint()
{
    _test_simpleser(1024, |i| i);
}

#[test]
fn test_uint_uint_uint()
{
    _test_simpleser(1024, |i| (i, (i+1, i-1)));
}

#[test]
fn test_vec_vec_uint()
{
    _test_simpleser(128, |_| vec![vec![0u, 1u], vec![1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2]]);
}

#[test]
fn test_option_uint()
{
    _test_simpleser(1024, |i| if i % 2 == 0 { Some(i) } else { None });
}


// bounces some elements back and forth between simple serialize stacks, encoding/decoding ...
fn _test_simpleser<T: SimpleSerialize<R>+Eq+PartialEq, R: SimpleSerializeVec<T>>(number: uint, element: |uint|:'static -> T)
{
    let mut stack1: R = Default::default();
    let mut stack2: R = Default::default();

    let mut buffers = Vec::new();

    for index in range(0, number) { stack1.push(element(index)); }
    stack1.encode(&mut buffers);

    for _ in range(0u, 10)
    {
        // decode, move, encode
        stack1.decode(&mut buffers);
        while let Some(record) = stack1.pop() { stack2.push(record); }
        stack2.encode(&mut buffers);

        // decode, move, encode
        stack2.decode(&mut buffers);
        while let Some(record) = stack2.pop() { stack1.push(record); }
        stack1.encode(&mut buffers);
    }

    stack1.decode(&mut buffers);
    for index in range(0, number)
    {
        if let Some(record) = stack1.pop()
        {
            // elements popped in reverse order from insert
            if record.ne(&element(number - index - 1))
            {
                panic!("un-equal elements found");
            }
        }
        else
        {
            panic!("Too few elements pop()d.");
        }
    }
}
