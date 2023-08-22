use crate::traits::numerical::CanRound;
use bigdecimal::BigDecimal;

#[test]
fn f32_round_2dp() {
    let n = 14.555f32;
    let result = n.round_2dp();
    println!("{} == {}", n, result);
    assert_eq!(result, 14.55f32);

    let n = 0.1599999f32;
    let result = n.round_2dp();
    println!("{} == {}", n, result);
    assert_eq!(result, 0.15f32);

    let n = 0.1f32;
    let result = n.round_2dp();
    println!("{} == {}", n, result);
    assert_eq!(result, 0.1);

    let n = 0.999f32;
    let result = n.round_2dp();
    println!("{} == {}", n, result);
    assert_eq!(result, 0.99);
}

#[test]
fn big_decimal_round_2dp() {
    let n = BigDecimal::from(14.555f32);
    let result = n.round_2dp();
    println!("{} == {}", n, result);
    assert_eq!(result, BigDecimal::from(14.55f32));

    let n = BigDecimal::from(0.1599999f32);
    let result = n.round_2dp();
    println!("{} == {}", n, result);
    assert_eq!(result, BigDecimal::from(0.15f32));

    let n = BigDecimal::from(0.1f32);
    let result = n.round_2dp();
    println!("{} == {}", n, result);
    assert_eq!(result, BigDecimal::from(0.1));

    let n = BigDecimal::from(0.999f32);
    let result = n.round_2dp();
    println!("{} == {}", n, result);
    assert_eq!(result, BigDecimal::from(0.99));
}
