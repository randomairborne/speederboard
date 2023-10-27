# templates

speederboard uses [tera](https://keats.github.io/tera/), which is similiar to Jinja2 and handlebars.

The handlers that render the templates definite a Rust struct which is serialized to tera- This context can be viewed at the top of the page html when in dev mode, with specifics available in the handler file context structs.

{% raw %}

```jinja2
{{ variable }}
{{ struct.variable }}
```

{% endraw %}

There are three functions worth knowing about. - `gettrans`, `getuserlinks`, and `getgamelinks`

## `gettrans`
`gettrans` takes at minimum two arguments, `key` and `lang`. 
You can set lang to the "magic" variable `language`, and `key`
should be a static string literal referencing the key in en.lang.

Some translations also support interpolation. Any extra arguments passed into gettrans will be available in that translation.

{% raw %}

```jinja2
{{ gettrans(lang=language, key="base.name") }}
```

```jinja2
{{ gettrans(lang=language, key="moderation.ban", name=user.username) }}
```

{% endraw %}

## `getuserlinks`
`getuserlinks` takes exactly one argument- a `User` object. It returns a map of strings: `pfp_url`, `banner_url`, `stylesheet_url`, and `ui_url`. For `stylesheet_url`, you must check if the user has a stylesheet before using it- but you should only ever need to use it in base.jinja. The others will always be valid.

{% raw %}

```jinja2
{% set links = getuserlinks(user=user) %}
{{ links.pfp_url }}
{{ links.banner_url }}
{{ links.stylesheet_url }}
{{ links.ui_url }}
```

{% endraw %}

## `getgamelinks`
`getuserlinks` takes exactly one argument- a `Game` object. It returns a map of strings: `cover_art_url`, `banner_url`, and `ui_url`.
These will always be valid links.

{% raw %}

```jinja2
{% set links = getgamelinks(game=game) %}
{{ links.cover_art_url }}
{{ links.banner_url }}
{{ links.ui_url }}
```

{% endraw %}

