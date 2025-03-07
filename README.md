A mix between grep and jq

---

⚠️ The query language is experimental, and bound to change. ⚠️

If you want to make suggestions, discussion is welcomed in the issues

- [Usage](#Usage)
  - [Query language](#Query-language)
  - [Flags](#Flags)
    - [Displaying JSON instead of the path](#Displaying-JSON-instead-of-the-path)
    - [Context](#Context)

## Usage
We will use the following JSON for all the examples:
```json
{
  "items": [
    {
      "id": 1,
      "name": "Lorem",
      "active": true,
      "meta": {
        "rating": 4.7,
        "author": { "name": "John", "verified": false }
      }
    },
    {
      "id": 2,
      "name": "Ipsum",
      "active": false,
      "meta": {
        "rating": 3.9,
        "author": { "name": "Jane", "verified": true }
      }
    }
  ]
}
```

### Query language
Jrep uses a very simple query language, based on the JSON path syntax, and admitting wildcards (*) for keys and values.
```bash
jrep 'Jane' filename
#> .items[1].meta.author.name: "Jane"
```

Will look for all the occurrences of 'Jane', in keys or values, and it will return the path to it.

You can also look for keys only, by putting the keys before a colon (:), or starting it with a dot (.):
```bash
jrep 'author:' filename
# or
jrep '.meta.author' filename
#> .items[0].meta.author: { 
#>   "name": "John", 
#>   "verified": false 
#> }
#> .items[1].meta.author: {
#>   "name": "Jane",
#>   "verified": true 
#> }
```

Or for values only, by putting the value after a colon:
```bash
jrep ': true' filename
#> .items[0].active: true
#> .items[1].meta.author.verified: true
```

Or for values and keys, having the key before a colon and the value after it:
```bash
jrep '.rating: 4.7' filename
#> .items[0].meta.rating: 4.7
```

And you can use wildcards:
```bash
jrep '.name: J*e' filename
#> .items[0].meta.author.name: "John"
#> .items[1].meta.author.name: "Jane"
```

Wildcards work with numbers too:
```bash
jrep '.rating: 4.*' filename
#> .items[0].meta.rating: 4.7
```

### Flags
#### Displaying JSON instead of the path
If you want to see the whole matched json, not just the path to the matched part, use the ``--json`` (``-j``) flag.
```bash
jrep '.rating' filename -j
#> {
#>   "items": [
#>     {
#>       "meta": {
#>         "rating": 4.7
#>       }
#>     },
#>     {
#>       "meta": {
#>         "rating": 3.9
#>       }
#>     },
#>   ]
#> }
```

#### Context
Like for grep, ``--context`` (``-C``) displays information around the match. In jrep, each context prints one previous level of the JSON object.
```bash
jrep 'Jane' filename -C 1
#> .items[1].meta.author: {
#>   "name": "Jane", 
#>   "verified": true 
#> }
```
#### Ignore case
You can use the ``--ignore-case`` (``-i``) flag to ignore the case of the query.
```bash
jrep 'jane' filename -i
#> .items[1].meta.author.name: "Jane"
```
