# gen_docs_template

> **This CLI automatically creates templates of comment text required when generating docs.rs!!(docs.rs の生成時に必要なコメント文のテンプレートを自動作成する CLI です!!)**

# How to use

## install

Run the following command:

```
cargo install gen_docs_template
```

## run

It generates docs.rs template comments for all files in src.

```
gen_docs_template
```

| Before                                         | After                                         |
| ---------------------------------------------- | --------------------------------------------- |
| <img src="img/example-before.png" width="300"> | <img src="img/example-after.png" width="300"> |

[Crates.io](https://crates.io/crates/gen_docs_template)

# Arguments

You can use these arguments.

| Arguments | description                                                         | Example                      |
| --------- | ------------------------------------------------------------------- | ---------------------------- |
| none      | Generates docs.rs template comments for all files in src.           | gen_docs_template            |
| -path     | Generates docs.rs template comments for all files in `hoge`.        | gen_docs_template -path hoge |
| hard      | Execute comment granting ignoring the history of previous comments. | gen_docs_template hard       |
