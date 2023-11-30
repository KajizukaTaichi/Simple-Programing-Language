use crate::executor::{input, ExecutionMode, Executor, Type};

impl<'a> Executor<'a> {
    /// 標準出力
    pub fn print(&mut self, arg: String) {
        let mut text = String::new();
        self.log_print(format!("標準出力に表示します"));

        let value = self.compute(arg.trim().to_string());
        text += &self.type_string(value);
        text = text.replace("'", "").replace('"', "");

        if let ExecutionMode::Script = self.execution_mode {
            println!("{text}");
        } else {
            println!("[出力]: {text}");
        }
    }

    /// 標準入力
    pub fn input(&mut self, prompt: String) -> String {
        let prompt = match self.compute(prompt) {
            Type::String(s) => s,
            _ => {
                self.log_print("エラー! 入力プロンプトは文字列型です".to_string());
                "".to_string()
            }
        };
        if let ExecutionMode::Script = self.execution_mode {
            input(prompt.as_str())
        } else {
            self.log_print("標準入力を受け取ります".to_string());
            self.log_print(format!("プロンプト:「{prompt}」"));
            input("[入力]> ")
        }
    }

    /// 文字列型に変換
    pub fn string(&mut self, arg: String) -> String {
        self.log_print("文字列型に変換します".to_string());
        return match self.compute(arg) {
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

    /// 数値型に変換
    pub fn number(&mut self, arg: String) -> f64 {
        self.log_print("数値型に変換します".to_string());
        return match self.compute(arg) {
            Type::Number(i) => i,
            Type::List(l) => match &l[0] {
                Type::Number(i) => *i,
                Type::String(ls) => ls.parse().unwrap_or(0.0),
                _ => 0.0,
            },
            Type::String(s) => match s.parse() {
                Ok(i) => i,
                Err(_) => {
                    self.log_print("エラー! 変換できませんでした".to_string());
                    0.0
                }
            },
            Type::Bool(b) => {
                if b {
                    1.0
                } else {
                    0.0
                }
            }
        };
    }

    /// 論理型に変換
    pub fn bool(&mut self, arg: String) -> bool {
        self.log_print("論理型に変換します".to_string());
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

    /// リストを生成
    pub fn list(&mut self, arg: String) -> Vec<Type> {
        let mut list: Vec<Type> = Vec::new();
        for i in self.tokenize_arguments(arg.as_str()) {
            if i.trim().is_empty() {
                continue;
            }
            list.push(self.compute(i.to_string()))
        }
        return list;
    }

    /// 変数を参照
    pub fn refer(&mut self, args: String) -> f64 {
        self.log_print("変数の参照を取得します".to_string());

        let address = self.reference_variable(args.clone());
        if let Some(i) = address {
            self.log_print(format!("変数{}のアドレスは{}です", args, i));
            i as f64
        } else {
            self.log_print(format!("エラー! 変数が見つかりませんでした"));
            0.0
        }
    }

    /// データ型を返す
    pub fn types(&mut self, args: String) -> Type {
        Type::String(match self.compute(args) {
            Type::Number(_) => "number".to_string(),
            Type::String(_) => "string".to_string(),
            Type::Bool(_) => "bool".to_string(),
            Type::List(_) => "list".to_string(),
        })
    }

    /// 指定したメモリアドレスにアクセス
    pub fn access(&mut self, args: String) -> Type {
        let address = match self.compute(args.clone()) {
            Type::Number(n) => n,
            _ => {
                self.log_print("エラー! メモリアドレスは数値型です".to_string());
                0.0
            }
        };
        self.log_print(format!("メモリアドレス{address}の指す値を求めます"));
        if address.round() as usize + 1 > self.memory.len() {
            println!("エラー! アドレスが有効範囲外です");
            Type::Number(0.0)
        } else {
            self.memory[address.round() as usize].value.clone()
        }
    }
}
