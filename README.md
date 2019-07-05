# Fourier
Fourier - 静的型付き純粋関数型プログラミング言語

Fourierは静的型付き純粋関数型プログラミング言語です。
型クラス、~~GADT、依存型~~などの機能があります。

## 目標
このプロジェクトでは以下を実現させることを目標とします。
- チューニングなしで、C言語以上の速度で実行されるバイナリを出力するコンパイラ
- MonadやGArrowなどの概念を利用した容易なEDSL定義による、ライブラリとしてのマルチパラダイム対応
- パラダイム間の自動翻訳（EDSL間トランスパイル）
- ~~強力な型システムと連携した~~周辺ツールによるプログラム検証の推進

## 進捗
構文解析器が80%程度、型チェッカが60%程度完成しています。
さまざまなバグ、未実装の機能があります。
### 未実装機能
型クラス、構造体定義、コード生成、標準ライブラリなど

## 構文
TODO
