# Windows环境

## 安装Rust

请跟随[官网指引](https://www.rust-lang.org/tools/install)，安装Rust。

## 将cream引入您的项目

创建一个新项目，并进入该目录

```shell
> cargo new my-first-app
> cd my-first-app
```

打开`Cargo.toml`文件，并在项目的`[dependencies]`下添加依赖。由于该库尚未正式发布，因此仅能通过git链接进行安装。

```toml
cream = { git = "https://github.com/Fancyflame/cream-rs.git" }
```

然后等待安装完成。

## 如果安装过程中`skia-safe`编译错误

有时因为各种原因，`skia-safe`无法正常编译。在本文中，推荐使用预编译包，这样您无需安装skia编译环境并等待编译。如无法下载预编译包，请检查网络连接是否正常。若仍无法下载，请采取手动下载方式，如下文。

1. 进入[`skia-binaries`](https://github.com/rust-skia/skia-binaries/releases)发布页。
2. 下载最新预编译包，格式为`skia-binaries-<编号>-<架构>-pc-windows-msvc-textlayout.tar.gz`,如`skia-binaries-4f106aa048fa92fce6ce-x86_64-pc-windows-msvc-textlayout.tar.gz`
3. 将压缩包复制到合适的文件夹，并选择下列任意一种方法。
   - 在系统搜索栏中搜索`PATH`，选择编辑系统环境变量，新建环境变量`SKIA_BINARIES_URL`，值为`<目录>/skia-binaries-{key}.tar.gz`，例如`C:\path\to\directory\skia-binaries-{key}.tar.gz`。注意，`{key}`需要保留。
   - 启动文件服务器[^1]，将上面步骤的值改为`<服务器地址>/skia-binaries-{key}.tar.gz`，例如`http://127.0.0.1:8000/skia-binaries-{key}.tar.gz`。
4. 重启您的控制台，或设置同样的环境变量，使得您的控制台能够识别您新设置的`SKIA_BINARIES_URL`环境变量。
5. 重新编译一次。

[^1]: 推荐选用nodejs或python文件服务器。如果您选用的是nodejs服务器，输入`http-server C:\path\to\directory -p 8000`来启动（需安装[`http-server`](https://www.npmjs.com/package/http-server)）。如果您选用的是python服务器，输入`python -m http.server -d C:\path\to\directory -p 8000`来启动。
