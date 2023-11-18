# templates

speederboard uses [tera](https://keats.github.io/tera/), which is similiar to Jinja2 and handlebars.

The handlers that render the templates definite a Rust struct which is serialized to tera- This context can be viewed at the top of the page html when in dev mode, with specifics available in the handler file context structs.

## Documentation for this documentation
Information on fields of structs is documented with a jinja comment. These look like {% raw %} {# this #} {% endraw %}.

### Link types

#### UI links
A "UI Link" is a link you can anchor, redirect, or type into the URL bar to bring someone to show them a nice HTML page
representing the documented resource. 

#### Action link
A link that can be specified as an `action` in an HTML form, taking an action on the inputted resource.

#### Image link
A link that can be loaded with an `img` tag.

### "Optional"
An optional variable may or may not be set, and must be checked for truthiness with 
{% raw %} 

```jinja2
{% if variable %} {% endif %}
```

{% endraw %}


### Variables
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
`getuserlinks` takes exactly one argument- a `User` object. 
It returns a map of strings, documented below. `stylesheet_url` is optional.

{% raw %}

```jinja2
{% set links = getuserlinks(user=user) %}
{{ links.pfp_url }}  {# Image link to this user's PFP #}
{{ links.banner_url }}  {# Image link to this user's banner #}
{{ links.stylesheet_url }}  {# (Optional) Resource link to this user's custom stylesheet. #}
{{ links.ui_url }} {# Link to this user's homepage on speederboard #}
```

{% endraw %}

## `getgamelinks`
`getgamelinks` takes exactly one argument- a `Game` object. 
It returns a map of strings, documented below.
These will always be valid links.

{% raw %}

```jinja2
{% set links = getgamelinks(game=game) %}
{{ links.cover_art_url }} {# Image link to this game's cover art #}
{{ links.banner_url }} {# Image link to this game's banner image #}
{{ links.ui_url }} {# UI link to this game's speederboard homepage #}
{{ links.edit_url }} {# UI link to this game's edit page #}
{{ links.feed_url }} {# UI link to this game's moderator feed #}
{{ links.team_url }} {# UI link to this game's moderator team #}
{{ links.forum_url }} {# UI link to this game's forum #}
{{ links.forum_new_post_url }} {# UI link to make a new post on this game's forum #}
```

{% endraw %}

## `getrunlinks`
`getrunlinks` takes exactly two arguments- a `Game` object, in `game`, and a `Run` object in `run`. 
It returns a map of strings, documented below.
These will always be valid links.

{% raw %}

```jinja2
{% set links = getrunlinks(game=game, run=run) %}
{{ links.review_url }} {# UI link to review the run #}
{{ links.verify_post_url }} {# Action link to verify a run #}
{{ links.reject_post_url }} {# Action link to reject a run #}
{{ links.ui_url }} {# UI link to view a run #}
```

{% endraw %}

## `getpostlinks`
`getpostlinks` takes exactly two arguments- a `Game` object, in `game`, and a `ForumPost` object in `post`.
A map of strings is returned, documented below.
These will always be valid links.

{% raw %}

```jinja2
{% set links = getpostlinks(game=game, run=run) %}
{{ links.ui_url }} {# UI link to this post #}
```

{% endraw %}

## `getcategorylinks`
`getcategorylinks` takes exactly two arguments- a `Game` object, in `game`, and a `MiniCategory` or `Category` object in `category`.
A map of strings is returned, documented below.
These will always be valid links.

{% raw %}

```jinja2
{% set links = getcategorylinks(game=game, category=category) %}
{{ links.ui_url }} {# UI link to this category #}
{{ links.feed_url }} {# UI link to the mod-feed for this category #}
{{ links.edit_url }} {# UI link to edit this category #}
{{ links.new_run_url }} {# UI link to make a new run in this category #}
```

{% endraw %}