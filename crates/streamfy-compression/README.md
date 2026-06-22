# Streamfy Compression

Library with handlers to compress and uncompress data in the streamfy protocol. 

In streamfy, compression is done in producer side, then consumers and SPU when it is using SmartModules, uncompress the data using the compression information that is in the attributes of the batch.

Currently, the supported compressions codecs are None (default), Gzip, Snappy, Zstd and LZ4.
