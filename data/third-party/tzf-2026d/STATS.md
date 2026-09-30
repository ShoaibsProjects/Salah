# Build Statistics

**Version:** 2026d-fix1
**Build date:** 2026-09-23T15:03:42Z

## Output Files

| File | Size | MD5 |
|------|------|-----|
| `lite.tzb` | 3.9 MB | `a943bc5e4bc3b45e98be03cfb92155e4` |
| `lite.tzm` | 9.8 MB | `21404de93d274c97cb5b4c7ea2d37daf` |
| `full.tzb` | 14.6 MB | `ec2f6ca6d4fb19d1029d582a1945d290` |

## Pipeline: `full.tzb` (dedup + compress on full precision)

### `deduplicatetzpb`

```
go: downloading github.com/ringsaturn/orb v0.15.0
go: downloading github.com/tidwall/rtree v1.11.1
go: downloading github.com/tidwall/geoindex v1.7.0
input:  timezones=444 polygons=1355 holes=791 points=8254651 bytes=121755885
output: shared_edges=3589 shared_points=4212403 inline_segs=183844 edge_ref_segs=5696 bytes=74555546
reduction: bytes=38.77%
dedup_rate: 3.01% of segments reference shared edges
```

### `compresstopotzpb`

```
input:  bytes=74555546
output: bytes=26057835
reduction: bytes=65.05%
```

### `topo2embed -profile e`

```
../tzf-dist/full.tzb
```

## Pipeline: `lite.tzb` (topology-aware simplify + dedup + compress + preindex)

### `reducetzpb -topology=true`

```
mode: topology
epsilon: 0.001000
dataset_before: timezones=444 polygons=1355 holes=791 points=8254651 bytes=121755878
dataset_after:  timezones=444 polygons=1355 holes=791 points=1129866 bytes=16677349
dataset_reduction: points=86.31% bytes=86.30%
topology_rings: total=2146 no_fixed=1518 one_fixed=8 multi_fixed=616 fallback=68 hole_escape=0 resimplified=40
topology_points: input=8251080 snapped_inserted=98 fallback_points=10107 fixed_vertices=187523
topology_segments: total=189037 shared=5445(2.88%) skipped_short=183793(97.23%) skipped_small=208(0.11%) cache_hits=2702 cache_misses=2743 cache_hit_rate=49.62%
topology_segment_points: input=8440105 output=1307039 reduction=84.51%
topology_segment_length_buckets: le10=184417 le25=438 le50=450 le100=567 gt100=3165
```

### `deduplicatetzpb`

```
input:  timezones=444 polygons=1355 holes=791 points=1129866 bytes=16677356
output: shared_edges=2537 shared_points=476833 inline_segs=183086 edge_ref_segs=4972 bytes=18780082
reduction: bytes=-12.61%
dedup_rate: 2.64% of segments reference shared edges
```

### `compresstopotzpb`

```
input:  bytes=18780082
output: bytes=13886678
reduction: bytes=26.06%
```

### `preindextzpb`

```
go: downloading golang.org/x/sync v0.22.0
input:  timezones=444 bytes=16677356
params: idxZoom=13 aggZoom=3 maxZoomLevelToKeep=10 layerDrop=2
output: total_keys=87736 bytes=2089619
```

### `topo2embed -profile e -preindex`

```
../tzf-dist/lite.tzb
```

## Pipeline: `lite.tzm` (memory-image transcode of lite.tzb)

### `tzb2tzm`

```
../tzf-dist/lite.tzm
```

