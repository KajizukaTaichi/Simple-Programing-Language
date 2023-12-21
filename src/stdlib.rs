use crate::executor::{input, ExecutionMode, Executor, ReturnValue, Type};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

impl<'a> Executor<'a> {
    /// 標準出力
    pub fn print(&mut self, arg: String) -> ReturnValue {
        let mut text = String::new();
        self.log_print(format!("標準出力に表示します"));

        let value = match self.compute(arg.trim().to_string()) {
            ReturnValue::Some(i) => i,
            ReturnValue::Error(e) => return ReturnValue::Error(e),
            _ => Type::Number(0.0),
        };
        text += &self.type_string(value);
        text = text.replace("'", "").replace('"', "");

        if let ExecutionMode::Script = self.execution_mode {
            println!("{text}");
        } else {
            println!("[出力]: {text}");
        }
        ReturnValue::None
    }

    /// 標準入力
    pub fn input(&mut self, prompt: String) -> ReturnValue {
        let prompt = match match self.compute(prompt) {
            ReturnValue::Some(i) => i,
            ReturnValue::Error(e) => return ReturnValue::Error(e),
            _ => Type::Number(0.0),
        } {
            Type::String(s) => s,
            _ => {
                self.log_print("エラー! 入力プロンプトは文字列型です".to_string());
                return ReturnValue::Error("エラー! 入力プロンプトは文字列型です".to_string());
            }
        };
        ReturnValue::Some(if let ExecutionMode::Script = self.execution_mode {
            Type::String(input(prompt.as_str()))
        } else {
            self.log_print("標準入力を受け取ります".to_string());
            self.log_print(format!("プロンプト:「{prompt}」"));
            Type::String(input("[入力]> "))
        })
    }

    pub fn time_now(&mut self) -> ReturnValue {
        self.log_print("今の時刻をUNIXエポックで取得します".to_string());
        let current_time = SystemTime::now();
        match current_time.duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let secs = duration.as_secs() as f64; // 秒数をf64に変換
                let nanos = duration.subsec_nanos() as f64; // ナノ秒をf64に変換
                let total_seconds = secs + nanos / 1_000_000_000.0; // 秒数とナノ秒の合計を計算

                ReturnValue::Some(Type::Number(total_seconds))
            }
            Err(err) => {
                self.log_print(format!("エラー: {:?}", err));
                return ReturnValue::Error(format!("エラー: {:?}", err));
            }
        }
    }

    pub fn time_sleep(&mut self, arg: String) -> ReturnValue {
        let sep = match match self.compute(arg) {
            ReturnValue::Some(i) => i,
            ReturnValue::Error(e) => return ReturnValue::Error(e),
            _ => Type::Number(0.0),
        } {
            Type::Number(i) => i,
            _ => {
                self.log_print("エラー! 秒数は数値型です".to_string());
                return ReturnValue::Error("エラー! 秒数は数値型です".to_string());
            }
        };
        self.log_print(format!("{sep}秒間スリープします"));
        thread::sleep(Duration::from_secs(sep as u64)); // 3秒間スリープ
        ReturnValue::None
    }

    /// 文字列型に変換
    pub fn string(&mut self, arg: String) -> ReturnValue {
        self.log_print("文字列型に変換します".to_string());
        return ReturnValue::Some(Type::String(
            match match self.compute(arg) {
                ReturnValue::Some(i) => i,
                ReturnValue::Error(e) => return ReturnValue::Error(e),
                _ => Type::Number(0.0),
            } {
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
            },
        ));
    }

    /// 数値型に変換
    pub fn number(&mut self, arg: String) -> ReturnValue {
        self.log_print("数値型に変換します".to_string());
        return ReturnValue::Some(Type::Number(
            match match self.compute(arg) {
                ReturnValue::Some(i) => i,
                ReturnValue::Error(e) => return ReturnValue::Error(e),
                _ => Type::Number(0.0),
            } {
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
                        return ReturnValue::Error("エラー! 変換できませんでした".to_string());
                    }
                },
                Type::Bool(b) => {
                    if b {
                        1.0
                    } else {
                        0.0
                    }
                }
            },
        ));
    }

    /// 論理型に変換
    pub fn bool(&mut self, arg: String) -> ReturnValue {
        self.log_print("論理型に変換します".to_string());
        ReturnValue::Some(Type::Bool(
            match match self.compute(
                arg[..arg.len() - 1].split("(").collect::<Vec<&str>>()[1..]
                    .join("(")
                    .to_string(),
            ) {
                ReturnValue::Some(i) => i,
                ReturnValue::Error(e) => return ReturnValue::Error(e),
                _ => Type::Number(0.0),
            } {
                Type::Number(i) => i != 0.0,
                Type::List(l) => l.len() != 0,
                Type::String(s) => !s.is_empty(),
                Type::Bool(b) => b,
            },
        ))
    }

    /// リストを生成
    pub fn list(&mut self, arg: String) -> ReturnValue {
        let mut list: Vec<Type> = Vec::new();
        for i in self.tokenize_arguments(arg.as_str()) {
            if i.trim().is_empty() {
                continue;
            }
            list.push(match self.compute(i.to_string()) {
                ReturnValue::Some(i) => i,
                ReturnValue::Error(e) => return ReturnValue::Error(e),
                _ => Type::Number(0.0),
            })
        }
        return ReturnValue::Some(Type::List(list));
    }

    /// 変数を参照
    pub fn refer(&mut self, args: String) -> ReturnValue {
        self.log_print("変数の参照を取得します".to_string());

        let address = self.reference_variable(args.clone());
        if let Some(i) = address {
            self.log_print(format!("変数{}のアドレスは{}です", args, i));
            return ReturnValue::Some(Type::Number(i as f64));
        } else {
            return ReturnValue::Error("エラー! 変数が見つかりませんでした".to_string());
        }
    }

    /// データ型を返す
    pub fn types(&mut self, args: String) -> ReturnValue {
        self.log_print(format!("データ型を判定します"));
        ReturnValue::Some(Type::String(
            match match self.compute(args) {
                ReturnValue::Some(i) => i,
                ReturnValue::Error(e) => return ReturnValue::Error(e),
                _ => Type::Number(0.0),
            } {
                Type::Number(_) => "number".to_string(),
                Type::String(_) => "string".to_string(),
                Type::Bool(_) => "bool".to_string(),
                Type::List(_) => "list".to_string(),
            },
        ))
    }

    /// 指定したメモリアドレスにアクセス
    pub fn access(&mut self, args: String) -> ReturnValue {
        let address = match match self.compute(args.clone()) {
            ReturnValue::Some(i) => i,
            ReturnValue::Error(e) => return ReturnValue::Error(e),
            _ => Type::Number(0.0),
        } {
            Type::Number(n) => n,
            _ => {
                self.log_print("エラー! メモリアドレスは数値型です".to_string());
                return ReturnValue::Error("エラー! メモリアドレスは数値型です".to_string());
            }
        };
        self.log_print(format!("メモリアドレス{address}の指す値を求めます"));
        if address.round() as usize + 1 > self.memory.len() {
            self.log_print("エラー! アドレスが有効範囲外です".to_string());
            return ReturnValue::Error("エラー! アドレスが有効範囲外です".to_string());
        } else {
            ReturnValue::Some(self.memory[address.round() as usize].value.clone())
        }
    }
}
