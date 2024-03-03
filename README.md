# ilytix
A simple command-line tool for visual image analysis, with features like checking image integrity, deduplication, and retrieval, written in Rust.

# Installation
```bash
pip install -U ilytix
# or
cargo install ilytix
```

# Getting Started

## Check the integrity of images (检查图片完整性)
It will attempt to repair incorrect image formats whenever possible.  
```bash
ilytix check -i ./datasets -r -o A/B/C
```

**Options:**  
`-i <PATH>` Path for input image or images folder.  
`-o <PATH>` Path for setting the saving results.  
`-r`, `--recursive` Recursively traverse folders to obtain files.  
`--mv` Store results by moving instead of copying.

**And you'll see something like this**
```bash
✔  Source · /home/qweasd/Desktop/datasets › Folder
✔  Recursively · true

🐢 Integrity Checking [####################] 73/73 (100% | 0.00s | 00:00:00)
✔  Found · x73
    · Intact › x34
    · Incorrect › x3
    · Deprecated Or Unsupported › x36

🐢 Saving[Copy] [####################] 73/73 (100% | 0.00s | 00:00:00)
✔  Results saved at · /home/qweasd/Desktop/A/B/C
```

## Images deduplication (图片去重)
Used for deduplicating images within a folder.  

```bash
ilytix dedup -i ./datasets -r -o A/B/C
```
**Options:**  
`-i <PATH>` Path for images folder.  
`-o <PATH>` Path for setting the saving results.  
`-r`, `--recursive` Recursively traverse folders to obtain files.  
`--mv` Store results by moving instead of copying.  
`-thresh` Used to adjust image similarity threshold.  

**And you'll see something like this**
```bash
✔  Source · /home/qweasd/Desktop/datasets › Folder
✔  Recursively · true

🐢 Building [####################] 73/73 (100% | 0.00s | 00:00:00)
✔  Index
    · Capacity › 73
    · Size › 37
    · Dimensions › 32

🐢 Deduplicating [####################] 73/73 (100% | 0.00s | 00:00:00)
✔  Found
    · Duplicated › x17
    · Curated › x20
    · Deprecated Or Unsupported › x36

🐢 Saving[Copy] [####################] 73/73 (100% | 0.00s | 00:00:00)
✔  Results saved at · /home/qweasd/Desktop/A/B/C-1

```

## Image-Image Retrival (图片检索相似图片)
```bash
ilytix retrive -i <Images Folder> -r --query <Query> -v
```

**Output**
```bash
✔  Collection · /home/qweasd/Desktop/4test › Folder
✔  Recursively · true

🐢 Building [####################] 10,693/10,693 (100% | 0.00s | 00:00:36)
✔  Index
    · Capacity › 10693
    · Size › 10677
    · Dimensions › 32
✔  Query · /home/qweasd/Desktop/query.png
✔  Matched · x6
    · /home/qweasd/Desktop/4test/clips/query (3rd copy).png
    · /home/qweasd/Desktop/4test/clips/query (another copy).png
    · /home/qweasd/Desktop/4test/clips/query (4th copy).png
    · /home/qweasd/Desktop/4test/clips/query (copy).png
    · /home/qweasd/Desktop/4test/clips/query.png
    · /home/qweasd/Desktop/4test/clips/query (5th copy).png

🐢 Saving[Copy] [####################] 6/6 (100% | 0.00s | 00:00:00)
✔  Results saved at · /home/qweasd/Desktop/A/B

```


# TODO
- [X]  images integrity check
- [X]  images de-duplicate
- [X]  image-image retrival
- [ ]  text-image retrival
- [ ]  image catption
