MESI: every line is Modified, Exclusive, Shared or Invalid. Writing to a Shared line means invalidating every other copy first, which is why two cores hammering one line is slow.

See [[cpu-architecture/caches/false-sharing]].
