fn kadane(array: &Vec<i32>) -> usize {
   let mut max_so_far = 0;
   let mut max_current = 0;

    for i in array {
        max_current += i;
        if max_current < 0 {
            max_current = 0;
        }
        if max_current > max_so_far {
            max_so_far = max_current;
        }
    }

    max_so_far as usize
}

fn kadane_find_subarray(array: &Vec<i32>) -> Vec<i32> {
    let mut max_so_far = 0;
    let mut max_current = 0;
    let mut buffer = Vec::new();
    let mut max_subarray = Vec::new();

    for i in array {
        buffer.push(i);
        max_current += i;
        if max_current < 0 {
            buffer.clear();
            max_current = 0;
        }
        if max_current > max_so_far {
            max_so_far = max_current;
            buffer.drain(0..buffer.len()).into_iter().for_each(|n| max_subarray.push(*n));
        }
    }

    max_subarray

}

fn main() {

}

mod test {
    use crate::{kadane, kadane_find_subarray};

    #[test]
    fn test_kadane() {
        assert_eq!(7, kadane(&vec![-2, -3, 4, -1, -2, 1, 5, -3]));
        assert_eq!(vec![4, -1, -2, 1, 5], kadane_find_subarray(&vec![-2, -3, 4, -1, -2, 1, 5, -3]))
    }
}