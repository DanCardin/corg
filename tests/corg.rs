use corg::{Corg, CorgError};

#[test]
fn default() -> Result<(), CorgError> {
    let result = Corg::default().execute("")?;
    assert_eq!(result, "");
    Ok(())
}

#[test]
fn warn_if_no_blocks() {
    let result = Corg::default().warn_if_no_blocks(true).execute("").is_err();
    assert!(result);
}

#[test]
fn basic_output() -> Result<(), CorgError> {
    let input = "
[[[#!bash
echo 1
]]]
[[[end]]]";
    let result = Corg::default().execute(input)?;

    let expected = "
[[[#!bash
echo 1
]]]
1
[[[end]]]
";

    assert_eq!(expected, result);
    Ok(())
}

#[test]
fn delete_blocks() -> Result<(), CorgError> {
    let input = "
[[[#!bash
echo 1
]]]
[[[end]]]";
    let result = Corg::default().delete_blocks(true).execute(input)?;
    let expected = "\n1\n";

    assert_eq!(expected, result);
    Ok(())
}

#[test]
fn two_blocks() -> Result<(), CorgError> {
    let input = "
[[[#!bash
echo 1
]]]
[[[end]]]
+
[[[#!bash
echo 2
]]]
[[[end]]]
";
    let result = Corg::default().delete_blocks(true).execute(input)?;
    let expected = "\n1\n+\n2\n";

    assert_eq!(expected, result);
    Ok(())
}

#[test]
fn omit_output() -> Result<(), CorgError> {
    let input = "
[[[#!bash
echo 1
]]]
[[[end]]]";
    let result = Corg::default().omit_output(true).execute(input)?;
    let expected = "
[[[#!bash
echo 1
]]]
[[[end]]]
";

    assert_eq!(expected, result);
    Ok(())
}

#[test]
fn check_only() {
    let input = "
[[[#!bash
echo 1
]]]
[[[end]]]";
    let result = Corg::default().check_only(true).execute(input).is_err();
    assert!(result);
}
