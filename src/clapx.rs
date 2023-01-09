use std::{path::PathBuf};

use minifb::Scale;

// pub static existing_file: Box<dyn TypedValueParser<Value = PathBuf> + Send + Sync> = Box::new(PathBufValueParser::new().try_map(ensure_existing_file));
// pub static existing_file: TypedValueParser<Value = PathBuf> = PathBufValueParser::new().try_map(ensure_existing_file);

// pub fn existing_file(value: &OsStr) -> anyhow::Result<PathBuf> {
//     PathBufValueParser::new().try_map(ensure_existing_file).parse_ref(cmd, arg, value)
// }


pub fn ensure_existing_file(p: PathBuf) -> anyhow::Result<PathBuf> {
    if !p.exists() {anyhow::bail!("Path does not exist")} else
    if !p.is_file() {anyhow::bail!("Path is not a file")} else
    {Ok(p)}
}

pub static SCALE_VALUES: [&'static str; 8]  = [
    "x1",
    "x2",
    "x4",
    "x8",
    "x16",
    "x32",
    "fit",
    "auto",
];

pub fn scale_value_parser(value: String) -> anyhow::Result<Scale> {
    match value.as_str() {
        "x1" => Ok(Scale::X1),
        "x2" => Ok(Scale::X2),
        "x4" => Ok(Scale::X4),
        "x8" => Ok(Scale::X8),
        "x16" => Ok(Scale::X16),
        "x32" => Ok(Scale::X32),
        "fit" | "auto" => Ok(Scale::FitScreen),
        _ => unreachable!("invalid value should be caught by PossibleValuesParser")
    }
}

// #[derive(Clone)]
// struct ExistingFileValueParser;

// impl clap::builder::TypedValueParser for ExistingFileValueParser {
//     type Value = Custom;
 
//     fn parse_ref(
//         &self,
//         cmd: &clap::Command,
//         arg: Option<&clap::Arg>,
//         value: &std::ffi::OsStr,
//     ) -> Result<Self::Value, clap::Error> {
//         let inner = clap::value_parser!(PathBuf);
//         let val = inner.parse_ref(cmd, arg, value)?;
 
//         let Err(err_m) = ensure_existing_file(val) else {
//             return Ok(val);
//         };

//         let mut err = clap::Error::new(ErrorKind::ValueValidation)
//                 .with_cmd(cmd);
//         if let Some(arg) = arg {
//             err.insert(ContextKind::InvalidArg, ContextValue::String(arg.to_string()));
//         }
            
//             err.insert(ContextKind::InvalidValue, ContextValue::String(INVALID_VALUE.to_string()));
//             return Err(err);
//         }
 
//         Ok(Custom(val))
//     }
//  }