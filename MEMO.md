# rapt

Toy version of apt written in apt.  

# cache file
aptのキャッシュファイルは`/var/cache/apt/pkgcache.bin`.
```.sh
$ file /var/cache/apt/pkgcache.bin
/var/cache/apt/pkgcache.bin: APT cache data, version 16.0, little-endian, 121553 packages, 120925 versions
```

# ソースリスト
ソースリストは`/etc/sources.list`もしくは`/etc/sources.list.d`以下のファイル。`update`時にこれらのリストのリポジトリからメタデータをfetchしてきて`/var/cache/apt/pkgccache.bin`に入れる。  
フォーマット: https://people.debian.org/~jak/apt2-doc/apt-Cache-Format.html
