use anyhow::Result;

use std::i64;

use crate::ops::RoundMode::*;
use crate::*;

type FixedPoint = crate::FixedPoint<i64, typenum::U9>;

// TODO: use `fixnum!(..)`.
fn fp(s: &str) -> Result<FixedPoint> {
    FixedPoint::from_str(s).map_err(From::from)
}

#[test]
fn from_decimal() -> Result<()> {
    let p1 = fp("5")?;
    let p2 = FixedPoint::from_decimal(5_000_000_000, -9);
    assert_eq!(Ok(p1), p2);

    Ok(())
}

#[test]
fn from_less_accurate_decimal() -> Result<()> {
    assert_eq!(FixedPoint::from_decimal(1, 0), Ok(fp("1")?));
    assert_eq!(FixedPoint::from_decimal(1, 1), Ok(fp("10")?));
    Ok(())
}

#[test]
fn from_good_str() -> Result<()> {
    assert_eq!(fp("1")?, fp("1.000000000")?);
    assert_eq!(fp("1.1")?, fp("1.100000000")?);
    assert_eq!(fp("1.02")?, fp("1.020000000")?);
    assert_eq!(fp("-1.02")?, fp("-1.020000000")?);
    assert_eq!(fp("+1.02")?, fp("1.020000000")?);
    assert_eq!(fp("123456789.123456789")?, fp("123456789.123456789")?);
    assert_eq!(fp("9223372036.854775807")?, fp("9223372036.854775807")?);
    assert_eq!(fp("0.1234")?, fp("0.123400000")?);
    assert_eq!(fp("-0.1234")?, fp("-0.123400000")?);

    Ok(())
}

#[test]
fn display() -> Result<()> {
    assert_eq!(format!("{}", fp("10.042")?), String::from("10.042"));
    assert_eq!(format!("{}", fp("10.042000")?), String::from("10.042"));
    assert_eq!(format!("{}", fp("-10.042")?), String::from("-10.042"));
    assert_eq!(format!("{}", fp("-10.042000")?), String::from("-10.042"));
    assert_eq!(
        format!("{}", fp("0.000000001")?),
        String::from("0.000000001")
    );
    assert_eq!(
        format!("{}", fp("-0.000000001")?),
        String::from("-0.000000001")
    );
    assert_eq!(format!("{}", fp("-0.000")?), String::from("0.0"));

    Ok(())
}

#[test]
fn from_bad_str() {
    let bad = &[
        "",
        "7.02e5",
        "a.12",
        "12.a",
        "13.0000000001",
        "13.1000000001",
        "13.9999999999999999999999999999999999999999999999999999999999999",
        "100000000000000000000000",
        "9223372036.854775808",
        "170141183460469231731687303715.884105728",
    ];

    for str in bad {
        assert!(fp(str).is_err(), "must not parse '{}'", str);
    }
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn exp_and_coef_should_agree() {
    assert!(FixedPoint::PRECISION > 0);
    assert_eq!(FixedPoint::COEF, 10i64.pow(FixedPoint::PRECISION as u32));
}

#[test]
fn cmul_overflow() {
    let result = FixedPoint::MAX.cmul(i64::MAX);
    assert_eq!(result, Err(ArithmeticError::Overflow));

    let result = FixedPoint::MAX.cmul(i64::MIN);
    assert_eq!(result, Err(ArithmeticError::Overflow));
}

macro_rules! assert_rmul {
    ($a:expr, $b:expr, $mode:ident, $result:expr) => {{
        let a = FixedPoint::try_from($a)?;
        let b = FixedPoint::try_from($b)?;

        // Check the commutative property.
        assert_eq!(a.rmul(b, $mode), b.rmul(a, $mode));
        // Check the result.
        assert_eq!(a.rmul(b, $mode), Ok(FixedPoint::try_from($result)?));
    }};
}

// TODO(hrls): remove
macro_rules! assert_rmuls {
    ($a:expr, $b:expr, $mode:ident, $result:expr) => {{
        let a = fp(&format!("{}", $a))?;
        let b = fp(&format!("{}", $b))?;

        // Check the commutative property.
        assert_eq!(a.rmul(b, $mode), b.rmul(a, $mode));
        // Check the result.
        assert_eq!(a.rmul(b, $mode), Ok(fp(&format!("{}", $result))?));
    }};
}

#[test]
#[allow(clippy::cognitive_complexity)]
fn rmul_exact() -> Result<()> {
    assert_rmul!(525, 10, Ceil, 5250);
    assert_rmul!(525, 10, Floor, 5250);
    assert_rmul!(-525, 10, Ceil, -5250);
    assert_rmul!(-525, 10, Floor, -5250);
    assert_rmul!(-525, -10, Ceil, 5250);
    assert_rmul!(-525, -10, Floor, 5250);
    assert_rmul!(525, -10, Ceil, -5250);
    assert_rmul!(525, -10, Floor, -5250);
    assert_rmuls!(525, "0.0001", Ceil, "0.0525");
    assert_rmuls!(525, "0.0001", Floor, "0.0525");
    assert_rmuls!(-525, "0.0001", Ceil, "-0.0525");
    assert_rmuls!(-525, "0.0001", Floor, "-0.0525");
    assert_rmuls!(-525, "-0.0001", Ceil, "0.0525");
    assert_rmuls!(-525, "-0.0001", Floor, "0.0525");
    assert_rmul!(FixedPoint::MAX, 1, Ceil, FixedPoint::MAX);
    assert_rmul!(FixedPoint::MAX, 1, Floor, FixedPoint::MAX);
    assert_rmuls!(
        FixedPoint::from_bits(i64::MAX / 10 * 10),
        "0.1",
        Ceil,
        FixedPoint::from_bits(i64::MAX / 10)
    );
    assert_rmuls!(
        FixedPoint::from_bits(i64::MAX / 10 * 10),
        "0.1",
        Floor,
        FixedPoint::from_bits(i64::MAX / 10)
    );
    assert_rmuls!(1, "0.000000001", Ceil, "0.000000001");
    assert_rmuls!(1, "0.000000001", Floor, "0.000000001");
    assert_rmuls!(-1, "-0.000000001", Ceil, "0.000000001");
    assert_rmuls!(-1, "-0.000000001", Floor, "0.000000001");

    Ok(())
}

#[test]
fn rmul_round() -> Result<()> {
    assert_rmuls!("0.1", "0.000000001", Ceil, "0.000000001");
    assert_rmuls!("0.1", "0.000000001", Floor, 0);
    assert_rmuls!("-0.1", "0.000000001", Ceil, 0);
    assert_rmuls!("-0.1", "0.000000001", Floor, "-0.000000001");
    assert_rmuls!("-0.1", "-0.000000001", Ceil, "0.000000001");
    assert_rmuls!("-0.1", "-0.000000001", Floor, 0);
    assert_rmuls!("0.000000001", "0.000000001", Ceil, "0.000000001");
    assert_rmuls!("0.000000001", "0.000000001", Floor, 0);
    assert_rmuls!("-0.000000001", "0.000000001", Ceil, 0);
    assert_rmuls!("-0.000000001", "0.000000001", Floor, "-0.000000001");

    Ok(())
}

#[test]
fn rmul_overflow() -> Result<()> {
    let a = FixedPoint::MAX;
    let b = fp("1.1")?;
    assert_eq!(a.rmul(b, Ceil), Err(ArithmeticError::Overflow));

    let a = fp("140000")?;
    assert_eq!(a.rmul(a, Ceil), Err(ArithmeticError::Overflow));

    let a = fp("-140000")?;
    let b = fp("140000")?;
    assert_eq!(a.rmul(b, Ceil), Err(ArithmeticError::Overflow));

    Ok(())
}

#[test]
fn rdiv_exact() -> Result<()> {
    let (numer, denom) = (fp("5")?, fp("2")?);
    let result = fp("2.5")?;
    assert_eq!(numer.rdiv(denom, Ceil), Ok(result));
    assert_eq!(numer.rdiv(denom, Floor), Ok(result));

    let (numer, denom) = (fp("-5")?, fp("2")?);
    let result = fp("-2.5")?;
    assert_eq!(numer.rdiv(denom, Ceil), Ok(result));
    assert_eq!(numer.rdiv(denom, Floor), Ok(result));

    let (numer, denom) = (fp("-5")?, fp("-2")?);
    let result = fp("2.5")?;
    assert_eq!(numer.rdiv(denom, Ceil), Ok(result));
    assert_eq!(numer.rdiv(denom, Floor), Ok(result));

    let (numer, denom) = (fp("5")?, fp("-2")?);
    let result = fp("-2.5")?;
    assert_eq!(numer.rdiv(denom, Ceil), Ok(result));
    assert_eq!(numer.rdiv(denom, Floor), Ok(result));

    let (numer, denom) = (FixedPoint::MAX, FixedPoint::MAX);
    let result = fp("1")?;
    assert_eq!(numer.rdiv(denom, Ceil), Ok(result));
    assert_eq!(numer.rdiv(denom, Floor), Ok(result));

    let (numer, denom) = (fp("5")?, fp("0.2")?);
    let result = fp("25")?;
    assert_eq!(numer.rdiv(denom, Ceil), Ok(result));
    assert_eq!(numer.rdiv(denom, Floor), Ok(result));

    let (numer, denom) = (fp("0.00000001")?, fp("10")?);
    let result = fp("0.000000001")?;
    assert_eq!(numer.rdiv(denom, Ceil), Ok(result));
    assert_eq!(numer.rdiv(denom, Floor), Ok(result));

    let (numer, denom) = (fp("0.00000001")?, fp("0.1")?);
    let result = fp("0.0000001")?;
    assert_eq!(numer.rdiv(denom, Ceil), Ok(result));
    assert_eq!(numer.rdiv(denom, Floor), Ok(result));

    Ok(())
}

#[test]
fn rdiv_i64() -> Result<()> {
    fn assert_rdiv(a: &str, b: i64, mode: RoundMode, expected: &str) -> Result<()> {
        let a = fp(a)?;
        let expected = fp(expected)?;
        assert_eq!(a.rdiv(b, mode).unwrap(), expected);
        Ok(())
    }

    assert_rdiv("2.4", 2, Ceil, "1.2")?;
    assert_rdiv("7", 3, Floor, "2.333333333")?;
    assert_rdiv("7", 3, Ceil, "2.333333334")?;
    assert_rdiv("-7", 3, Floor, "-2.333333334")?;
    assert_rdiv("-7", 3, Ceil, "-2.333333333")?;
    assert_rdiv("-7", -3, Floor, "2.333333333")?;
    assert_rdiv("-7", -3, Ceil, "2.333333334")?;
    assert_rdiv("7", -3, Floor, "-2.333333334")?;
    assert_rdiv("7", -3, Ceil, "-2.333333333")?;
    assert_rdiv("0", 5, Ceil, "0")?;
    assert_rdiv("0.000000003", 2, Ceil, "0.000000002")?;
    assert_rdiv("0.000000003", 2, Floor, "0.000000001")?;
    assert_rdiv("0.000000003", 7, Floor, "0")?;
    assert_rdiv("0.000000003", 7, Ceil, "0.000000001")?;
    assert_rdiv("0.000000001", 7, Ceil, "0.000000001")?;
    Ok(())
}

#[test]
fn rdiv_round() -> Result<()> {
    let (numer, denom) = (fp("100")?, fp("3")?);
    let ceil = fp("33.333333334")?;
    let floor = fp("33.333333333")?;
    assert_eq!(numer.rdiv(denom, Ceil), Ok(ceil));
    assert_eq!(numer.rdiv(denom, Floor), Ok(floor));

    let (numer, denom) = (fp("-100")?, fp("3")?);
    let ceil = fp("-33.333333333")?;
    let floor = fp("-33.333333334")?;
    assert_eq!(numer.rdiv(denom, Ceil), Ok(ceil));
    assert_eq!(numer.rdiv(denom, Floor), Ok(floor));

    let (numer, denom) = (fp("-100")?, fp("-3")?);
    let ceil = fp("33.333333334")?;
    let floor = fp("33.333333333")?;
    assert_eq!(numer.rdiv(denom, Ceil), Ok(ceil));
    assert_eq!(numer.rdiv(denom, Floor), Ok(floor));

    let (numer, denom) = (fp("100")?, fp("-3")?);
    let ceil = fp("-33.333333333")?;
    let floor = fp("-33.333333334")?;
    assert_eq!(numer.rdiv(denom, Ceil), Ok(ceil));
    assert_eq!(numer.rdiv(denom, Floor), Ok(floor));

    Ok(())
}

#[test]
fn rdiv_division_by_zero() {
    assert_eq!(
        FixedPoint::MAX.rdiv(FixedPoint::ZERO, Ceil),
        Err(ArithmeticError::DivisionByZero)
    );
}

#[test]
fn rdiv_overflow() -> Result<()> {
    assert_eq!(
        FixedPoint::MAX.rdiv(fp("0.5")?, Ceil),
        Err(ArithmeticError::Overflow)
    );
    Ok(())
}

#[test]
fn float_mul() {
    let a = FixedPoint::try_from(525).unwrap();
    let b = FixedPoint::try_from(10).unwrap();
    assert_eq!(a.rmul(b, Ceil), Ok(FixedPoint::try_from(5250).unwrap()));

    let a = FixedPoint::try_from(525).unwrap();
    let b = FixedPoint::from_str("0.0001").unwrap();
    assert_eq!(a.rmul(b, Ceil), Ok(FixedPoint::from_str("0.0525").unwrap()));

    let a = FixedPoint::MAX;
    let b = FixedPoint::try_from(1).unwrap();
    assert_eq!(a.rmul(b, Ceil), Ok(FixedPoint::MAX));

    let a = FixedPoint::from_bits(i64::MAX / 10 * 10);
    let b = FixedPoint::from_str("0.1").unwrap();
    assert_eq!(a.rmul(b, Ceil), Ok(FixedPoint::from_bits(i64::MAX / 10)));
}

#[test]
fn float_mul_overflow() {
    let a = FixedPoint::try_from(140_000).unwrap();
    assert!(a.rmul(a, Ceil).is_err());

    let a = FixedPoint::try_from(-140_000).unwrap();
    let b = FixedPoint::try_from(140_000).unwrap();
    assert!(a.rmul(b, Ceil).is_err());
}

#[test]
fn half_sum() -> Result<()> {
    fn t(a: &str, b: &str, r: &str) -> Result<()> {
        let a = fp(a)?;
        let b = fp(b)?;
        let r = fp(r)?;
        assert_eq!(FixedPoint::half_sum(a, b), r);
        Ok(())
    }

    t("1", "3", "2")?;
    t("1", "2", "1.5")?;
    t("9000", "9050", "9025")?;
    t("9000", "-9000", "0")?;
    t("9000000000", "9000000002", "9000000001")?;
    t(
        "9000000000.000000001",
        "-9000000000.000000005",
        "-0.000000002",
    )?;
    t("7.123456789", "7.123456788", "7.123456788")?;

    Ok(())
}

#[test]
#[allow(clippy::many_single_char_names)]
fn integral() -> Result<()> {
    let a = fp("0.0001")?;
    assert_eq!(a.integral(Floor), 0);
    assert_eq!(a.integral(Ceil), 1);

    let b = fp("-0.0001")?;
    assert_eq!(b.integral(Floor), -1);
    assert_eq!(b.integral(Ceil), 0);

    let c = FixedPoint::ZERO;
    assert_eq!(c.integral(Floor), 0);
    assert_eq!(c.integral(Ceil), 0);

    let d = fp("2.0001")?;
    assert_eq!(d.integral(Floor), 2);
    assert_eq!(d.integral(Ceil), 3);

    let e = fp("-2.0001")?;
    assert_eq!(e.integral(Floor), -3);
    assert_eq!(e.integral(Ceil), -2);

    Ok(())
}

#[test]
fn round_towards_zero_by() -> Result<()> {
    let a = fp("1234.56789")?;
    assert_eq!(a.round_towards_zero_by(fp("100")?), fp("1200")?);
    assert_eq!(a.round_towards_zero_by(fp("10")?), fp("1230")?);
    assert_eq!(a.round_towards_zero_by(fp("1")?), fp("1234")?);
    assert_eq!(a.round_towards_zero_by(fp("0.1")?), fp("1234.5")?);
    assert_eq!(a.round_towards_zero_by(fp("0.01")?), fp("1234.56")?);
    assert_eq!(a.round_towards_zero_by(fp("0.001")?), fp("1234.567")?);
    assert_eq!(a.round_towards_zero_by(fp("0.0001")?), fp("1234.5678")?);
    assert_eq!(a.round_towards_zero_by(fp("0.00001")?), fp("1234.56789")?);

    let b = fp("-1234.56789")?;
    assert_eq!(b.round_towards_zero_by(fp("100")?), fp("-1200")?);
    assert_eq!(b.round_towards_zero_by(fp("10")?), fp("-1230")?);
    assert_eq!(b.round_towards_zero_by(fp("1")?), fp("-1234")?);
    assert_eq!(b.round_towards_zero_by(fp("0.1")?), fp("-1234.5")?);
    assert_eq!(b.round_towards_zero_by(fp("0.01")?), fp("-1234.56")?);
    assert_eq!(b.round_towards_zero_by(fp("0.001")?), fp("-1234.567")?);
    assert_eq!(b.round_towards_zero_by(fp("0.0001")?), fp("-1234.5678")?);
    assert_eq!(b.round_towards_zero_by(fp("0.00001")?), fp("-1234.56789")?);

    Ok(())
}

#[test]
#[allow(clippy::cognitive_complexity)]
fn next_power_of_ten() -> Result<()> {
    assert_eq!(fp("0")?.next_power_of_ten()?, fp("0.000000001")?);
    assert_eq!(fp("0.000000001")?.next_power_of_ten()?, fp("0.000000001")?);
    assert_eq!(fp("0.000000002")?.next_power_of_ten()?, fp("0.00000001")?);
    assert_eq!(fp("0.000000009")?.next_power_of_ten()?, fp("0.00000001")?);
    assert_eq!(fp("0.00000001")?.next_power_of_ten()?, fp("0.00000001")?);
    assert_eq!(fp("0.00000002")?.next_power_of_ten()?, fp("0.0000001")?);
    assert_eq!(fp("0.1")?.next_power_of_ten()?, fp("0.1")?);
    assert_eq!(fp("0.100000001")?.next_power_of_ten()?, fp("1")?);
    assert_eq!(fp("1")?.next_power_of_ten()?, fp("1")?);
    assert_eq!(fp("2")?.next_power_of_ten()?, fp("10")?);
    assert_eq!(fp("1234567")?.next_power_of_ten()?, fp("10000000")?);
    assert_eq!(
        fp("923372036.854775807")?.next_power_of_ten()?,
        fp("1000000000")?
    );
    assert_eq!(
        fp("9223372036.854775807")?.next_power_of_ten(),
        Err(ArithmeticError::Overflow)
    );

    assert_eq!(
        fp("-0.000000001")?.next_power_of_ten()?,
        fp("-0.000000001")?
    );
    assert_eq!(fp("-0.000000002")?.next_power_of_ten()?, fp("-0.00000001")?);
    assert_eq!(fp("-0.000000009")?.next_power_of_ten()?, fp("-0.00000001")?);
    assert_eq!(fp("-0.00000001")?.next_power_of_ten()?, fp("-0.00000001")?);
    assert_eq!(fp("-0.00000002")?.next_power_of_ten()?, fp("-0.0000001")?);
    assert_eq!(fp("-0.1")?.next_power_of_ten()?, fp("-0.1")?);
    assert_eq!(fp("-0.2")?.next_power_of_ten()?, fp("-1")?);
    assert_eq!(fp("-1")?.next_power_of_ten()?, fp("-1")?);
    assert_eq!(fp("-0.100000001")?.next_power_of_ten()?, fp("-1")?);
    assert_eq!(fp("-1234567")?.next_power_of_ten()?, fp("-10000000")?);
    assert_eq!(
        fp("-923372036.854775808")?.next_power_of_ten()?,
        fp("-1000000000")?
    );
    assert_eq!(
        fp("-9223372036.854775807")?.next_power_of_ten(),
        Err(ArithmeticError::Overflow)
    );
    assert_eq!(
        fp("-9223372036.854775808")?.next_power_of_ten(),
        Err(ArithmeticError::Overflow)
    );

    Ok(())
}

#[test]
fn rounding_to_i64() {
    fn t(x: &str, r: i64) {
        let f = FixedPoint::from_str(x).unwrap();
        assert_eq!(f.rounding_to_i64(), r);
    }

    t("0", 0);
    t("42", 42);
    t("1.4", 1);
    t("1.6", 2);
    t("-1.4", -1);
    t("-1.6", -2);
    t("0.4999", 0);
    t("0.5", 1);
    t("0.5001", 1);
}

#[test]
#[cfg(feature = "std")]
fn to_f64() {
    fn t(x: &str, expected: f64) {
        let f = FixedPoint::from_str(x).unwrap();
        let actual = f.to_f64();
        assert_eq!(actual.to_string(), expected.to_string());
    }

    t("0", 0.0);
    t("1", 1.0);
    t("1.5", 1.5);
    t("42.123456789", 42.123_456_789);
    t("-14.14", -14.14);
    t("8003332421.536753168", 8_003_332_421.536_754);
}

#[test]
fn saturating_add() -> Result<()> {
    fn t(a: &str, b: &str, expected: &str) -> Result<()> {
        let a: FixedPoint = a.parse()?;
        let b: FixedPoint = b.parse()?;
        let expected: FixedPoint = expected.parse()?;
        assert_eq!(a.saturating_add(b), expected);
        assert_eq!(b.saturating_add(a), expected);
        assert_eq!(a.cneg()?.saturating_add(b.cneg()?), expected.cneg()?);
        Ok(())
    }

    t("0", "0", "0")?;
    t("0", "3000.0000006", "3000.0000006")?;
    t("-1000.0000002", "0", "-1000.0000002")?;
    t("-1000.0000002", "3000.0000006", "2000.0000004")?;
    t("-1000.0000002", "-3000.0000006", "-4000.0000008")?;
    t(
        "4611686018.427387903",
        "4611686018.427387903",
        "9223372036.854775806",
    )?;
    assert_eq!(
        fp("9222222222")?.saturating_add(fp("9222222222")?),
        FixedPoint::MAX
    );
    assert_eq!(
        fp("4611686019")?.saturating_add(fp("4611686018.427387903")?),
        FixedPoint::MAX
    );
    assert_eq!(
        fp("-9222222222")?.saturating_add(fp("-9222222222")?),
        FixedPoint::MIN
    );
    assert_eq!(
        fp("-4611686019")?.saturating_add(fp("-4611686018.427387903")?),
        FixedPoint::MIN
    );
    Ok(())
}

#[test]
fn saturating_mul() -> Result<()> {
    fn t(a: &str, b: i64, expected: &str) -> Result<()> {
        let a: FixedPoint = a.parse()?;
        let expected: FixedPoint = expected.parse()?;
        assert_eq!(a.saturating_mul(b), expected);
        assert_eq!(CheckedMul::saturating_mul(b, a), expected);
        assert_eq!(a.cneg()?.saturating_mul(b), expected.cneg()?);
        assert_eq!(a.saturating_mul(-b), expected.cneg()?);
        assert_eq!(a.cneg()?.saturating_mul(-b), expected);
        Ok(())
    }

    t("0", 0, "0")?;
    t("3000.0000006", 0, "0")?;
    t("3000.0000006", 1, "3000.0000006")?;
    t("-1000.0000002", 0, "0")?;
    t("-1000.0000002", 3, "-3000.0000006")?;
    t("-1000.0000002", -4, "4000.0000008")?;
    t("68601.48179", -468, "-32105493.47772")?;
    assert_eq!(
        fp("9222222222")?.saturating_mul(9222222222),
        FixedPoint::MAX
    );
    assert_eq!(
        CheckedMul::saturating_mul(4611686019, fp("4611686018.427387903")?),
        FixedPoint::MAX
    );
    assert_eq!(
        fp("-9222222222")?.saturating_mul(9222222222),
        FixedPoint::MIN
    );
    assert_eq!(
        CheckedMul::saturating_mul(4611686019, fp("-4611686018.427387903")?),
        FixedPoint::MIN
    );
    Ok(())
}

#[test]
fn saturating_rmul() -> Result<()> {
    fn t(a: &str, b: &str, expected: &str) -> Result<()> {
        let a: FixedPoint = a.parse()?;
        let b: FixedPoint = b.parse()?;
        let expected: FixedPoint = expected.parse()?;
        assert_eq!(a.saturating_rmul(b, Floor), expected);
        assert_eq!(b.saturating_rmul(a, Floor), expected);
        assert_eq!(a.cneg()?.saturating_rmul(b, Floor), expected.cneg()?);
        assert_eq!(a.saturating_rmul(b.cneg()?, Floor), expected.cneg()?);
        assert_eq!(a.cneg()?.saturating_rmul(b.cneg()?, Floor), expected);
        Ok(())
    }

    t("0", "0", "0")?;
    t("0", "3000.0000006", "0")?;
    t("1", "3000.0000006", "3000.0000006")?;
    t("-1000.0000002", "0", "0")?;
    t("-1000.0000002", "3", "-3000.0000006")?;
    t("-1000.0000002", "-4", "4000.0000008")?;
    t("68601.48179", "-468.28", "-32124701.8926212")?;
    assert_eq!(
        fp("0.000000001")?.saturating_rmul(fp("0.1")?, Floor),
        fp("0")?
    );
    assert_eq!(
        fp("0.000000001")?.saturating_rmul(fp("-0.1")?, Floor),
        fp("-0.000000001")?
    );
    assert_eq!(
        fp("0.000000001")?.saturating_rmul(fp("0.1")?, Ceil),
        fp("0.000000001")?
    );
    assert_eq!(
        fp("-0.000000001")?.saturating_rmul(fp("0.1")?, Ceil),
        fp("0")?
    );
    assert_eq!(
        fp("9222222222")?.saturating_rmul(fp("9222222222")?, Floor),
        FixedPoint::MAX
    );
    assert_eq!(
        fp("4611686019")?.saturating_rmul(fp("4611686018.427387903")?, Floor),
        FixedPoint::MAX
    );
    assert_eq!(
        fp("-9222222222")?.saturating_rmul(fp("9222222222")?, Floor),
        FixedPoint::MIN
    );
    assert_eq!(
        fp("4611686019")?.saturating_rmul(fp("-4611686018.427387903")?, Floor),
        FixedPoint::MIN
    );
    Ok(())
}

#[test]
fn saturating_sub() -> Result<()> {
    fn t(a: &str, b: &str, expected: &str) -> Result<()> {
        let a: FixedPoint = a.parse()?;
        let b: FixedPoint = b.parse()?;
        let expected: FixedPoint = expected.parse()?;
        assert_eq!(a.saturating_sub(b), expected);
        assert_eq!(b.saturating_sub(a), expected.cneg()?);
        assert_eq!(a.cneg()?.saturating_sub(b.cneg()?), expected.cneg()?);
        Ok(())
    }

    t("0", "0", "0")?;
    t("0", "3000.0000006", "-3000.0000006")?;
    t("-1000.0000002", "0", "-1000.0000002")?;
    t("-1000.0000002", "3000.0000006", "-4000.0000008")?;
    t("-1000.0000002", "-3000.0000006", "2000.0000004")?;
    t(
        "4611686018.427387903",
        "-4611686018.427387903",
        "9223372036.854775806",
    )?;
    assert_eq!(
        fp("9222222222")?.saturating_sub(fp("-9222222222")?),
        FixedPoint::MAX
    );
    assert_eq!(
        fp("4611686019")?.saturating_sub(fp("-4611686018.427387903")?),
        FixedPoint::MAX
    );
    assert_eq!(
        fp("-9222222222")?.saturating_sub(fp("9222222222")?),
        FixedPoint::MIN
    );
    assert_eq!(
        fp("-4611686019")?.saturating_sub(fp("4611686018.427387903")?),
        FixedPoint::MIN
    );
    Ok(())
}
