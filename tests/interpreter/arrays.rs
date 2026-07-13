use super::helpers::run;

#[test]
fn array_decl_and_index() {
    assert!(run("void main() { int[] arr = [10, 20, 30]; int x = arr[1]; }").is_ok());
}

#[test]
fn array_element_assignment() {
    assert!(run("void main() { int[] arr = [1, 2, 3]; arr[0] = 99; int x = arr[0]; }").is_ok());
}

#[test]
fn nested_array_assignment() {
    assert!(run("void main() { int[] row0 = [1, 2]; int[] row1 = [3, 4]; int[][] matrix = [row0, row1]; matrix[1][0] = 99; }").is_ok());
}

#[test]
fn out_of_bounds() {
    let result = run("void main() { int[] arr = [1, 2]; int x = arr[5]; }");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("out of bounds"));
}
