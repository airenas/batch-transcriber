### File dir watcher
```mermaid

sequenceDiagram
    participant worker as Worker
    participant db as PostgreSQL
    participant fs as FS
    participant queue as Queue(PostgreSQL)

    worker ->> fs: get incoming files
    fs -->> worker: files(name, size, date)
    worker ->> db: get incomming files
    db -->> worker: files(name, size, date)

    alt new/updated files
        worker ->> db: save new/updated files to db
        db -->> worker: 
    end
    alt existing files
        worker ->> db: drop from db
        worker ->> fs: move to working
        worker ->> queue: add work queue
        queue -->> worker: 
    end
```

### ASR Queue Worker

```mermaid

sequenceDiagram
    participant queue as Queue(PostgreSQL)
    participant worker as Worker
    participant db as PostgreSQL
    participant fs as FS
    participant asr as ASR

    queue->>worker: msg
    worker ->>+ db: id get info
    db -->>- worker: 
    worker ->> asr: upload
    asr -->> worker: 
    worker ->>+ db: save
    db -->>- worker: 
    loop not finished
    worker ->> asr: status
    asr -->> worker: 
    worker ->> queue: status msg
    queue -->> worker: 
    end
    alt is failed
        worker ->> fs: move to failed
    else is ok
        worker ->> asr: result
        asr -->> worker: 
        worker ->> fs: move to processed
    end
    worker ->> asr: delete
    asr -->> worker: 

```


### Upload

```mermaid
sequenceDiagram
    participant browser as UploadGUI
    participant uploader as Uploader
    participant db as PostgreSQL
    participant fs as FS
    participant queue as Queue(PostgreSQL)

    browser  ->>+ uploader: save
    uploader ->>+ db: file
    db -->>- uploader: 
    uploader ->>+ fs: file
    fs -->>- uploader: 
    uploader ->>+ queue: add (id, file_dir)
    queue -->>- uploader: 
    uploader -->>- browser: 

```

### List

```mermaid
sequenceDiagram
    participant browser as ViewGUI
    participant viewer as Viewer
    participant db as PostgreSQL


    browser->>+ viewer: 
    viewer->>+ db: file
    db -->>- viewer: 
    viewer -->>- browser: 

```

### Open Transcription

```mermaid
sequenceDiagram
    participant browser as ViewGUI
    participant editor as Trans Editor
    participant viewer as Viewer
    participant db as PostgreSQL
    participant fs as fs

    browser->>+ editor: open in browser link
    editor ->>+ viewer: url
    viewer->>+ db: id
    db -->>- viewer: file
    viewer ->>+ fs: get file
    fs -->>- viewer: 
    viewer -->>- editor: 
```

### Components

1. UploadGUI
2. ViewGUI
3. Uploader
4. Viewer
5. Worker
6. DirWatcher
7. PostgreSQL
8. Queue(PostgreSQL)