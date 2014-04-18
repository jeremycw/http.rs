LDFLAGS=-L lib -L /usr/local/Cellar/pcre/8.34/lib
CFLAGS=
CC=rustc

all: http

http: http.rs
	$(CC) $(CFLAGS) $(LDFLAGS) http.rs
