# style语法

## 声明样式

声明样式的语法为

```text
<样式类型>: [<参数>, ..][.<选项> [<值>], ..];
```

<div class="tip">

为了方便起见，当一个样式在宏内是以snake_case方式[^1]被声明时，将被解析为CamelCase，然后在头部添加“Style”。例如“width”将被转义为“StyleWidth”。

</div>

[^1]: snake_case是一种命名规则，用于将多个单词连接成一个单词并用下划线分隔。在snake_case中，单词以小写字母表示，并使用下划线分隔，例如：my_variable_name。而在rust中，类型应该以CamelCase的命名方式出现。
