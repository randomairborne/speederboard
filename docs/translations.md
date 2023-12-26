# writing translations

Hello translator! Developing translations for speederboard is pretty easy.
The file format is quite basic, using a nested JSON dictionary of translation
keys which map to translation values. A dot is inserted at each level of indirection- for example:

```json
{
  "foo": "one",
  "bar": {
    "baz": "two",
    "quux": "{three}"
  }
}
```

In the above example, you would use `foo` to reference `one`, and `bar.baz` to reference `two`.

You can also use `{name}` to reference a variable. You can view the english file, which also
serves as a source of truth for the keys, [here](https://github.com/speederboard/speederboard/blob/main/translations/en.lang).
