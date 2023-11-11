use crate::executor::{input, ExecutionMode, Executor, Type};

impl<'a> Executor<'a> {
    pub fn print(&mut self, arg: String) {
        let mut text = String::new();
        self.log_print(format!("標準出力に表示します"));
        match self.compute(arg.trim().to_string()) {
            Type::Number(i) => {
                text += i.to_string().as_str();
            }
            Type::String(s) => {
                text += s.replace("'", "").replace('"', "").as_str();
            }
            Type::List(l) => {
                text += "[";
                for i in l {
                    match i {
                        Type::Number(i) => text += format!("{}, ", i).as_str(),
                        Type::String(s) => text += format!("'{}' , ", s).as_str(),
                        _ => {}
                    }
                }
                text += "]";
            }
            Type::Bool(b) => {
                text += &b.to_string();
            }
        }
        if let ExecutionMode::Script = self.execution_mode {
            println!("{text}");
        } else {
            println!("[出力]: {text}");
        }
    }

    /// 標準入力
    pub fn input(&mut self) -> String {
        let inputed = if let ExecutionMode::Script = self.execution_mode {
            input("> ")
        } else {
            println!("標準入力を受け取ります");
            input("[入力]> ")
        };
        inputed
    }

    pub fn string(&mut self, arg: String) -> String {
        return match self.compute(
            arg[..arg.len() - 1].split("(").collect::<Vec<&str>>()[1..]
                .join("(")
                .to_string(),
        ) {
            Type::Number(i) => i.to_string(),
            Type::List(l) => l
                .iter()
                .map(|x| match x {
                    Type::Number(i) => i.to_string(),
                    Type::String(s) => format!("'{s}'"),
                    Type::List(_) => "".to_string(),
                    Type::Bool(b) => b.to_string(),
                })
                .collect::<Vec<String>>()
                .join(", "),
            Type::String(s) => s,
            Type::Bool(b) => b.to_string(),
        };
    }

    pub fn number(&mut self, arg: String) -> f64 {
        return match self.compute(
            arg[..arg.len() - 1].split("(").collect::<Vec<&str>>()[1..]
                .join("(")
                .to_string(),
        ) {
            Type::Number(i) => i,
            Type::List(l) => match &l[0] {
                Type::Number(i) => *i,
                Type::String(ls) => ls.parse().unwrap_or(0.0),
                _ => 0.0,
            },
            Type::String(s) => s.parse().unwrap_or({
                println!("エラー! 変換できませんでした");
                0.0
            }),
            Type::Bool(b) => {
                if b {
                    1.0
                } else {
                    0.0
                }
            }
        };
    }

    pub fn bool(&mut self, arg: String) -> bool {
        match self.compute(
            arg[..arg.len() - 1].split("(").collect::<Vec<&str>>()[1..]
                .join("(")
                .to_string(),
        ) {
            Type::Number(i) => i != 0.0,
            Type::List(l) => l.len() != 0,
            Type::String(s) => !s.is_empty(),
            Type::Bool(b) => b,
        }
    }

    pub fn list(&mut self, arg: String) -> Vec<Type> {
        let expr = arg
            .replacen("list", "", 1)
            .replace("[", "")
            .replace("]", "");
        let mut list: Vec<Type> = Vec::new();
        for i in expr.split(",") {
            if i.trim().is_empty() {
                continue;
            }
            list.push(self.compute(i.to_string()))
        }
        return list;
    }
    pub fn refer(&mut self, args: String) -> f64 {
        self.log_print("変数の参照を取得します".to_string());
        let code = self
            .tokenize_arguments(
                args[..args.len() - 1]
                    .replacen("ref", "", 1)
                    .replacen("(", "", 1)
                    .as_str(),
            )
            .join(",");

        let address = self.reference_variable(code.clone());
        if let Some(i) = address {
            self.log_print(format!("変数{}のアドレスは{}です", code, i));
            i as f64
        } else {
            self.log_print(format!("エラー! 変数が見つかりませんでした"));
            0.0
        }
    }
}
