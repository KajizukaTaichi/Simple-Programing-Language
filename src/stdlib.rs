use crate::executor::{input, ExecutionMode, Executor, ReturnValue, Type};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

impl<'a> Executor<'a> {
    /// 標準出力
    pub fn print(&mut self, arg: Type) -> ReturnValue {
        let text = self.type_string(arg);
        self.log_print(format!("標準出力に表示します"));

        let text = text.replace("'", "").replace('"', "");

        if let ExecutionMode::Script = self.execution_mode {
            println!("{text}");
        } else {
            println!("[出力]: {text}");
        }
        ReturnValue::None
    }

    /// 標準入力
    pub fn input(&mut self, prompt: Type) -> ReturnValue {
        let prompt = match prompt {
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

    pub fn time_sleep(&mut self, arg: Type) -> ReturnValue {
        let sep = match arg {
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
    pub fn string(&mut self, arg: Type) -> ReturnValue {
        self.log_print("文字列型に変換します".to_string());
        return ReturnValue::Some(Type::String(match arg {
            Type::Number(i) => i.to_string(),
            Type::List(l) => l
                .iter()
                .map(|x| match x {
                    Type::Number(i) => i.to_string(),
                    Type::String(s) => format!("'{s}'"),
                    Type::List(_) => "".to_string(),
                    Type::Bool(b) => b.to_string(),
                    Type::Function(i) => self.function_propaty(*i).name.clone(),
                })
                .collect::<Vec<String>>()
                .join(", "),
            Type::String(s) => s,
            Type::Bool(b) => b.to_string(),
            Type::Function(i) => self.function_propaty(i).name.clone(),
        }));
    }

    /// 数値型に変換
    pub fn number(&mut self, arg: Type) -> ReturnValue {
        self.log_print("数値型に変換します".to_string());
        let value = ReturnValue::Some(Type::Number(match arg {
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
            Type::Function(i) => self.function_propaty(i).code.len() as f64,
        }));
        return value;
    }

    /// 論理型に変換
    pub fn bool(&mut self, arg: Type) -> ReturnValue {
        self.log_print("論理型に変換します".to_string());
        ReturnValue::Some(Type::Bool(match arg {
            Type::Number(i) => i != 0.0,
            Type::List(l) => l.len() != 0,
            Type::String(s) => !s.is_empty(),
            Type::Bool(b) => b,
            Type::Function(i) => self.function_propaty(i).code.len() != 0,
        }))
    }

    /// リストを生成
    pub fn list(&mut self) -> ReturnValue {
        return ReturnValue::Some(Type::List(vec![]));
    }

    /// 変数を参照
    pub fn refer(&mut self, args: Type) -> ReturnValue {
        self.log_print("変数の参照を取得します".to_string());

        let name = match args.clone() {
            Type::String(s) => s,
            _ => "".to_string(),
        };
        let address = self.reference_variable(name.clone());
        if let Some(i) = address {
            self.log_print(format!("変数{}のアドレスは{}です", name, i));
            return ReturnValue::Some(Type::Number(i as f64));
        } else {
            return ReturnValue::Error("エラー! 変数が見つかりませんでした".to_string());
        }
    }

    /// データ型を返す
    pub fn types(&mut self, args: Type) -> Type {
        self.log_print(format!("データ型を判定します"));
        Type::String(match args {
            Type::Number(_) => "number".to_string(),
            Type::String(_) => "string".to_string(),
            Type::Bool(_) => "bool".to_string(),
            Type::List(_) => "list".to_string(),
            Type::Function(_) => "function".to_string(),
        })
    }

    /// 指定したメモリアドレスにアクセス
    pub fn access(&mut self, args: Type) -> ReturnValue {
        let address = match args {
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

    /// 標準ライブラリを呼び出す
    pub fn call_stdlib(&mut self, name: String, args: Vec<Type>) -> ReturnValue {
        //　入力
        if name == "input" {
            self.log_print(format!("標準ライブラリのinput関数を呼び出します"));
            return self.input(args[0].clone());
        }

        //　出力
        if name == "print" {
            self.log_print(format!("標準ライブラリのprint関数を呼び出します"));
            return self.print(args[0].clone());
        }

        //　現在時刻
        if name == "time.now" {
            self.log_print(format!("標準ライブラリのtime.now関数を呼び出します"));
            return self.time_now();
        }

        //　スリープ
        if name == "time.sleep" {
            self.log_print(format!("標準ライブラリのtime.sleep関数を呼び出します"));
            return self.time_sleep(args[0].clone());
        }

        // 参照
        if name == "ref" {
            self.log_print(format!("標準ライブラリのref関数を呼び出します"));
            return self.refer(args[0].clone());
        }

        //　指定したメモリアドレスにアクセス
        if name == "access" {
            self.log_print(format!("標準ライブラリのaccess関数を呼び出します"));
            return self.access(args[0].clone());
        }

        // データ型を判定
        if name == "type" {
            self.log_print(format!("標準ライブラリのtype関数を呼び出します"));
            return ReturnValue::Some(self.types(args[0].clone()));
        }

        // 文字列に変換
        if name == "string" {
            self.log_print(format!("標準ライブラリのstring関数を呼び出します"));
            return self.string(args[0].clone());
        }

        // 数値に変換
        if name == "number" {
            self.log_print(format!("標準ライブラリのnumber関数を呼び出します"));
            return self.number(args[0].clone());
        }

        // 論理値に変換
        if name == "bool" {
            self.log_print(format!("標準ライブラリのbool関数を呼び出します"));
            return self.bool(args[0].clone());
        }

        // リストを作成
        if name == "list" {
            self.log_print(format!("標準ライブラリのlist関数を呼び出します"));
            return self.list();
        }

        return ReturnValue::None;
    }

    pub fn std_func_args_len(&mut self, name: String) -> usize {
        match name.as_str() {
            "input" => 1,
            "print" => 1,
            "time.now" => 0,
            "time.sleep" => 1,
            "ref" => 1,
            "access" => 1,
            "type" => 1,
            "string" => 1,
            "number" => 1,
            "bool" => 1,
            "list" => 0,
            _ => 0,
        }
    }
}
