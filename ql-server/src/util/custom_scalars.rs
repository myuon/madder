use juniper::{
    parser::{ParseError, ScalarToken, Token},
    ParseScalarResult, Value,
};

#[derive(Clone)]
pub struct ClockTime(gst::ClockTime);

impl ClockTime {
    pub fn new(time: gst::ClockTime) -> ClockTime {
        ClockTime(time)
    }
}

// lost information here, u64 -> i32
graphql_scalar!(ClockTime where Scalar = <S> {
    resolve(&self) -> Value {
        Value::scalar(self.0.mseconds().unwrap() as i32)
    }

    from_input_value(v: &InputValue) -> Option<ClockTime> {
        v.as_scalar_value::<i32>().map(|u| ClockTime(gst::ClockTime::from_mseconds(u.clone() as u64)))
    }

    from_str<'a>(value: ScalarToken<'a>) -> ParseScalarResult<'a, S> {
        if let ScalarToken::Int(value) = value {
            Ok(S::from(value.to_owned()))
        } else {
            Err(ParseError::UnexpectedToken(Token::Scalar(value)))
        }
    }
});

#[derive(Clone, Copy)]
pub struct U32(u32);

impl U32 {
    pub fn from_i32(v: i32) -> Self {
        U32(v as u32)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

graphql_scalar!(U32 where Scalar = <S> {
    resolve(&self) -> Value {
        Value::scalar(self.0 as i32)
    }

    from_input_value(v: &InputValue) -> Option<U32> {
        v.as_scalar_value::<i32>().map(|u| U32(u.clone() as u32))
    }

    from_str<'a>(value: ScalarToken<'a>) -> ParseScalarResult<'a, S> {
        if let ScalarToken::Int(value) = value {
            Ok(S::from(value.to_owned()))
        } else {
            Err(ParseError::UnexpectedToken(Token::Scalar(value)))
        }
    }
});
