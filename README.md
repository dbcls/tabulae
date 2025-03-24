# Tabulae: Build and share datasets for ML applications from SPARQL

# What is this?

Tabulae [tǽbjəliː], “tables” in Latin, is a tool for combining multiple SPARQL queries to create well-prepared tabular datasets for subsequent applications. In particular, it is intended to be used from the extensive ecosystem in the field of machine learning. Tabulae creates base tables from SPARQL queries (called Layer 1), and then combines and processes these tables in [DuckDB](https://duckdb.org/) to form new tables (called Layer 2). The output table dumps can be hosted statically on any HTTP(s) server, along with a web UI that allows users to browse a summary of tables. For convenience, dumps are provided in several formats. You can use it in spreadsheets, import it from Jupyter notebooks, or query it directly over the network using DuckDB.

# Getting started

## Overview

First of all, we need to create `queries` directory. This directory will contain two subfolders, named `layer1` and `layer2`. Place SPARQL queries in `layer1` and SQL queries for DuckDB in `layer2`. Layer 1 queries must have a `.rq` extension. Layer 2 queries must have a `.sql` extension. The file name without the suffix becomes the table name as it is. A typical directory structure is as follows:

```
queries
├── layer1
│   ├── chembl_compound-atc-classification.rq
│   ├── chembl_compound-uniprot-via-activity_assay.rq
│   └── uniprot_protein-location.rq
└── layer2
    └── combined.sql
```

And this is exactly what we are going to build in this tutorial.

## Defining Layer 1 Tables

We are going to create a SPARQL query. Here, we use the endpoint provided by [rdfportal.org](http://rdfportal.org/) to query the ChEMBL database for “atcClassification”, as follows:

```sparql
# chembl_compound-atc-classification.rq
# Endpoint: https://rdfportal.org/ebi/sparql
PREFIX cco: <http://rdf.ebi.ac.uk/terms/chembl#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX skos: <http://www.w3.org/2004/02/skos/core#>

SELECT DISTINCT ?chembl_compound_id ?label ?atc
WHERE {
    ?Molecule a cco:SmallMolecule ;
              skos:prefLabel ?label ;
              cco:atcClassification ?atc .
  BIND (STRAFTER(STR(?Molecule), "http://rdf.ebi.ac.uk/resource/chembl/molecule/") AS ?chembl_compound_id)
}
```

The `# Endpoint` declaration in the first line is a magic comment that Tabulae recognizes specially. A SPARQL query is issued to this endpoint. This declaration is required for all SPARQL queries.

The variable names of the bindings returned by the SELECT query become the column names of the generated table. In addition, for each column, the type is automatically inferred (if all SPARQL Results are the same type and the type is supported by Tabular). If not possible, the default is `VARCHAR`.

Let's create another query, as shown in the following list:

```sparql
# chembl_compound-uniprot-via-activity_assay.rq
# Endpoint: https://rdfportal.org/ebi/sparql
PREFIX cco: <http://rdf.ebi.ac.uk/terms/chembl#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX skos: <http://www.w3.org/2004/02/skos/core#>
PREFIX tax: <http://identifiers.org/taxonomy/>

SELECT DISTINCT
  ?chembl_compound_id
  ?uniprot_id
  ?conf_score
WHERE {
  ?Molecule a cco:SmallMolecule ;
      cco:hasActivity/cco:hasAssay [
        a cco:Assay ;
        cco:targetConfScore ?conf_score ;
        cco:hasTarget/skos:exactMatch [
        cco:taxonomy tax:9606 ;
        skos:exactMatch ?uniprot
        ]
      ] .
  ?uniprot a cco:UniprotRef .
  BIND (STRAFTER(STR(?Molecule), "http://rdf.ebi.ac.uk/resource/chembl/molecule/") AS ?chembl_compound_id)
  BIND (STRAFTER(STR(?uniprot), "http://purl.uniprot.org/uniprot/") AS ?uniprot_id)
}
# Paginate: 1000000
```

This query has the text `# Paginate: 1000000` at the end. This is also a magic comment. When this is specified, Tabulae parses the given query and obtains the entries while rewriting them with the number of entries given by OFFSET and LIMIT (1 million in this example). Please note that the magic comments just need to start at the beginning of a line, and can be placed on any line.

Let's add one more query, as follows. This is the last one:

```sparql
# uniprot_protein-location.rq
# Endpoint: https://rdfportal.org/sib/sparql
PREFIX core: <http://purl.uniprot.org/core/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX uniprot: <http://purl.uniprot.org/uniprot/>
PREFIX db: <http://purl.uniprot.org/database/>
PREFIX chu: <http://purl.uniprot.org/proteomes/UP000005640#Unplaced>
PREFIX ch: <http://purl.uniprot.org/proteomes/UP000005640#Chromosome%20>
PREFIX skos: <http://www.w3.org/2004/02/skos/core#>

SELECT DISTINCT ?uniprot_id ?tissue ?tissue_label
WHERE {
  VALUES ?human_proteome { ch:1 ch:2 ch:3 ch:4 ch:5 ch:6 ch:7 ch:8 ch:9 ch:10
                     ch:11 ch:12 ch:13 ch:14 ch:15 ch:16 ch:17 ch:18 ch:19
                     ch:20 ch:21 ch:22 ch:X ch:Y ch:MT chu: }

  ?uniprot core:proteome ?human_proteome ;
           core:isolatedFrom ?tissue .
  ?tissue skos:prefLabel ?tissue_label .

  BIND (STRAFTER(STR(?uniprot), "http://purl.uniprot.org/uniprot/") AS ?uniprot_id)
}
```

## Generating Layer 1 tables

We have created a query for the three tables for Layer 1. Let's try running it. In the example, we use podman to run the Docker container:

```
podman run --rm -it -v ./queries:/work/queries -v ./dist:/work/dist tabulae build
```

It may take a while. The query will be executed and the tables will be generated. The results will be placed under the `dist` directory.

The dist directory should be something like this:

```
dist
├── assets
│   ├── index-Duuvv_a3.js
│   ├── index--oYnr9KL.css
│   ├── sparql-ClhyTxQt.js
│   ├── sql-D8pCRXw6.js
│   └── wasm-CG6Dc4jp.js
├── index.html
├── layer1
│   ├── chembl_compound-atc-classification.csv
│   ├── chembl_compound-atc-classification.parquet
│   ├── chembl_compound-atc-classification.tsv
│   ├── chembl_compound-uniprot-via-activity_assay.csv
│   ├── chembl_compound-uniprot-via-activity_assay.parquet
│   ├── chembl_compound-uniprot-via-activity_assay.tsv
│   ├── uniprot_protein-location.csv
│   ├── uniprot_protein-location.parquet
│   └── uniprot_protein-location.tsv
├── layer1.duckdb
├── layer2
├── layer2.duckdb
└── manifest.json
```

It includes files for web front ends and dumps of each table format. There are also DuckDB database files.

## Publishing the tables

In order to check the results, we need to serve the contents under the dist directory as a static website using a web server. Here, we start an nginx container with Podman.

```
podman run -it --rm --name tabulae-nginx -v ./dist:/usr/share/nginx/html:ro -p 8080:80 docker.io/library/nginx
```

Open [localhost:8080](http://localhost:8080/) in your web browser. The Web UI should then be displayed.

The Web UI shows only the Layer 2 tables by default. You will need to turn on the “Show Layer 1 tables” switch in order to see the three Layer 1 tables you created earlier.

If you have updated, added or deleted queries, just re-run the above command. Tabulae checks the timestamp of the query files and only re-runs the queries that have been updated. (This is because Layer 1 tables usually take a long time to generate), while Layer 2 tables are always re-generated.

## Defining Layer 2 Table

Next, let's create a Layer 2 table by combining these three tables.

Layer 2 tables must have a .sql extension. Just remember that the first part of the file name will become the name of the table that is created.

Write the following query:

```sql
-- queries/layer2/combined.sql
SELECT
*
FROM 'chembl_compound-atc-classification'
NATURAL JOIN 'chembl_compound-uniprot-via-activity_assay'
NATURAL JOIN 'uniprot_protein-location';
```

This is where you write your DuckDB queries. In this context, you can see the Layer 1 tables (you can't see the Layer 2 tables, as this is to simplify the dependency management). The output table for the statement in this file will be a Layer 2 table. The column names and types will also be used as they are.

## Generating Layer 2 Table

Run `tabulae build` again:

```
podman run --rm -it -v ./queries:/work/queries -v ./dist:/work/dist tabulae build
```

Then, if you reload [localhost:8080](http://localhost:8080/), you should see the Layer 2 table you just created.

Here, you can use any query from DuckDB. Therefore, you can also publish data obtained from other data sources as Layer 2 tables, not just SPARQL.

Using Tabulae, you can integrate multiple tables like this to create and publish tables for specific applications.

# Consuming the tables

## Local use

If you want to keep the trial-and-error loop for building the Layer 2 table short, you can also try out queries directly using the DuckDB CLI. The Layer 1 table is included in dist/layer1.duckdb. So, if DuckDB CLI is installed, you can query the layer1 table in read-only mode as follows ('D ' is the DuckDB prompt):

```
❯ duckdb --readonly dist/layer1.duckdb
v1.2.0 5f5512b827
Enter ".help" for usage hints.
D SELECT * FROM 'chembl_compound-atc-classification' LIMIT 5;
┌────────────────────┬────────────┬─────────┐
│ chembl_compound_id │   label    │   atc   │
│      varchar       │  varchar   │ varchar │
├────────────────────┼────────────┼─────────┤
│ CHEMBL1027         │ TIAGABINE  │ N03AG06 │
│ CHEMBL1089         │ PHENELZINE │ N06AF03 │
│ CHEMBL115          │ INDINAVIR  │ J05AE02 │
│ CHEMBL11672        │ ROQUINIMEX │ L03AX02 │
│ CHEMBL1171837      │ PONATINIB  │ L01EA05 │
└────────────────────┴────────────┴─────────┘
```

If you have a good query, just place it as a file under queries/layer2/ and rebuild.

The Layer2 tables are also in dist/layer2.duckdb, so you can use them in the same way.

## Remote use

If you make it publicly available via HTTP(s), it can be used by various clients. You can also query it directly from DuckDB:

```
❯ duckdb
v1.2.0 5f5512b827
Enter ".help" for usage hints.
Connected to a transient in-memory database.
Use ".open FILENAME" to reopen on a persistent database.
D INSTALL httpfs;
D ATTACH 'http://localhost:8080/layer1.duckdb';
D USE layer1;
D SHOW TABLES;
┌────────────────────────────────────────────┐
│                    name                    │
│                  varchar                   │
├────────────────────────────────────────────┤
│ chembl_compound-atc-classification         │
│ chembl_compound-uniprot-via-activity_assay │
│ uniprot_protein-location                   │
└────────────────────────────────────────────┘
D SELECT * FROM 'chembl_compound-atc-classification' LIMIT 5;
┌────────────────────┬────────────┬─────────┐
│ chembl_compound_id │   label    │   atc   │
│      varchar       │  varchar   │ varchar │
├────────────────────┼────────────┼─────────┤
│ CHEMBL1027         │ TIAGABINE  │ N03AG06 │
│ CHEMBL1089         │ PHENELZINE │ N06AF03 │
│ CHEMBL115          │ INDINAVIR  │ J05AE02 │
│ CHEMBL11672        │ ROQUINIMEX │ L03AX02 │
│ CHEMBL1171837      │ PONATINIB  │ L01EA05 │
└────────────────────┴────────────┴─────────┘
```

Of course, you can query an individual table dump:

```
D SELECT * FROM 'http://localhost:8080/layer1/chembl_compound-atc-classification.parquet' LIMIT 5;
┌────────────────────┬────────────┬─────────┐
│ chembl_compound_id │   label    │   atc   │
│      varchar       │  varchar   │ varchar │
├────────────────────┼────────────┼─────────┤
│ CHEMBL1027         │ TIAGABINE  │ N03AG06 │
│ CHEMBL1089         │ PHENELZINE │ N06AF03 │
│ CHEMBL115          │ INDINAVIR  │ J05AE02 │
│ CHEMBL11672        │ ROQUINIMEX │ L03AX02 │
│ CHEMBL1171837      │ PONATINIB  │ L01EA05 │
└────────────────────┴────────────┴─────────┘

```

You can find these URLs from the Web UI.

# Notes on serving files via HTTP(s): about CORS

If you want to retrieve these tables via a browser, you must set the CORS headers appropriately. Note that DuckDB also has a WebAssembly build, which is required when using the tables from there.

Here is a simple configuration example based on [https://enable-cors.org/server_nginx.html](https://enable-cors.org/server_nginx.html). Please rewrite this to suit your needs:

```
user  nginx;
worker_processes  auto;

error_log  /var/log/nginx/error.log notice;
pid        /var/run/nginx.pid;

events {
    worker_connections  1024;
}

http {
    include       /etc/nginx/mime.types;
    default_type  application/octet-stream;

    log_format  main  '$remote_addr - $remote_user [$time_local] "$request" '
                      '$status $body_bytes_sent "$http_referer" '
                      '"$http_user_agent" "$http_x_forwarded_for"';

    access_log  /var/log/nginx/access.log  main;

    sendfile        on;
    #tcp_nopush     on;

    keepalive_timeout  65;

    gzip  on;

    server {
        listen       80;
	listen  [::]:80;
        server_name  localhost;
    
        location / {
             root   /usr/share/nginx/html;
             index  index.html index.htm;
             if ($request_method = 'OPTIONS') {
                add_header 'Access-Control-Allow-Origin' '*';
                add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS';
                #
                # Custom headers and headers various browsers *should* be OK with but aren't
                #
                add_header 'Access-Control-Allow-Headers' 'DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range';
                #
                # Tell client that this pre-flight info is valid for 20 days
                #
                add_header 'Access-Control-Max-Age' 1728000;
                add_header 'Content-Type' 'text/plain; charset=utf-8';
                add_header 'Content-Length' 0;
                return 204;
             }
             if ($request_method = 'POST') {
                add_header 'Access-Control-Allow-Origin' '*' always;
                add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS' always;
                add_header 'Access-Control-Allow-Headers' 'DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range' always;
                add_header 'Access-Control-Expose-Headers' 'Content-Length,Content-Range' always;
             }
             if ($request_method = 'GET') {
                add_header 'Access-Control-Allow-Origin' '*' always;
                add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS' always;
                add_header 'Access-Control-Allow-Headers' 'DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range' always;
                add_header 'Access-Control-Expose-Headers' 'Content-Length,Content-Range' always;
             }
        }
    
        error_page   500 502 503 504  /50x.html;
        location = /50x.html {
            root   /usr/share/nginx/html;
        }
    }
}

```

When starting with Podman, this configuration file must be enabled, as follows:

```
podman run -it --rm --name tabulae-nginx -v ./dist:/usr/share/nginx/html:ro -v ./nginx.conf:/etc/nginx/nginx.conf:ro -p 8080:80 docker.io/library/nginx
```
