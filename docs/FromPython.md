# Use published tables from Python

In order to access the published tables from a Python environment, it is convenient to use the Python package for duckdb. [Google Colab](https://colab.research.google.com/) comes with the duckdb package installed by default, so you can use it simply with `import duckdb`. If you want to use it in other environments, please refer to the [document](https://duckdb.org/docs/stable/clients/python/overview).

The following examples assume that they will be run in the Google colab environment, but they should work in other environments with small modifications. Please replace the URL of the DuckDB database with the appropriate one.

## Accessing the published tables

```python
import duckdb

con = duckdb.connect()

# Attach Layer 1 database and use it
con.sql("""
ATTACH 'https:/example.com/layer1.duckdb' (READ_ONLY);
USE layer1;
""")

# Show the tables
con.sql("SHOW TABLES;").show()

# Query
con.sql("""
SELECT * FROM 'chembl_compound-atc-classification' LIMIT 5;
""")
```

If you want to use the tables in the Layer 2 database, you can attach the Layer 2 database and use it in the same way.

```python
import duckdb

con = duckdb.connect()

# Attach Layer 2 database and use it
con.sql("""
ATTACH 'https:/example.com/layer2.duckdb' (READ_ONLY);
USE layer2;
""")

# Show the tables
con.sql("SHOW TABLES;").show()
```

## Transforming the table and train a model

You can also transform the table using SQL queries. The following example shows how to join three tables and extract the first letter of the ATC code. Then, it makes a 1-hot feature vector with the PIVOT operation. Calling `.to_df()` on the result will return a pandas DataFrame. Then, we will train the predictor using [AutoGluon](https://auto.gluon.ai/stable/index.html).

First of all, install the `autogluon` package:

```
!pip install autogluon
```

Then, run the following code:

```python
import duckdb

con = duckdb.connect()

# Attach Layer 1 database and use it
con.sql("""
ATTACH 'https:/example.com/layer1.duckdb' (READ_ONLY);
USE layer1;
""")

# Join three tables; The python variable `combined` can be referenced in the SQL query after this
# Omitting `SELECT *`; DuckDB allows this
combined = con.sql("""
FROM 'chembl_compound-atc-classification'
NATURAL JOIN 'chembl_compound-uniprot-via-activity_assay'
NATURAL JOIN 'uniprot_protein-tissue';
""")

# Extract the first letter of the ATC code
tissue_label_atc = con.sql("""
SELECT chembl_compound_id, tissue_label, atc[1] AS target FROM combined LIMIT 5;
""")

# Make 1-hot feature vectors with PIVOT
wide_table = con.sql("""
PIVOT
    (FROM tissue_label_atc)
ON tissue_label
USING COUNT(*) > 0;
""")

# Make DataFrame
df = wide_table.to_df()
```

Let's try training a model by treating this data frame as a classification problem.

Make the `TabularDataset` from the DataFrame, dropping the ID column:

```python
from autogluon.tabular import TabularDataset, TabularPredictor

train = TabularDataset(df.drop(columns=["chembl_compound_id"]))
```

Then train the model:

```python
predictor = TabularPredictor(label="target").fit(train)
```

Show the summary:

```python
predictor.fit_summary(show_prot=True)
```

# NetworkX example

We will calculate the number of hops in the protein-protein interaction. We will use the Uniprot protein-protein interaction data from the Layer 1 database.

Save the following SPARQL query as a layer1 query and generate a table:

```sparql
# queries/layer1/uniprot_protein-protein.rq
# Endpoint: https://rdfportal.org/sib/sparql
PREFIX core: <http://purl.uniprot.org/core/>
PREFIX owl: <http://www.w3.org/2002/07/owl#>
PREFIX uniprot: <http://purl.uniprot.org/uniprot/>
PREFIX chu: <http://purl.uniprot.org/proteomes/UP000005640#Unplaced>
PREFIX ch: <http://purl.uniprot.org/proteomes/UP000005640#Chromosome%20>
SELECT DISTINCT ?uniprot_id1 ?uniprot_id2
FROM <http://sparql.uniprot.org/uniprot>
FROM <http://sparql.uniprot.org/keywords>
WHERE {
  VALUES ?human_proteome { ch:1 ch:2 ch:3 ch:4 ch:5 ch:6 ch:7 ch:8 ch:9 ch:10
                     ch:11 ch:12 ch:13 ch:14 ch:15 ch:16 ch:17 ch:18 ch:19
                     ch:20 ch:21 ch:22 ch:X ch:Y ch:MT chu: }

  ?uniprot core:proteome ?human_proteome ;
           core:interaction [
             a core:Non_Self_Interaction ;
             core:participant/owl:sameAs ?uniprot2
           ] .
  ?uniprot2 core:proteome ?human_proteome .

  BIND (STRAFTER(STR(?uniprot), "http://purl.uniprot.org/uniprot/") AS ?uniprot_id1)
  BIND (STRAFTER(STR(?uniprot2), "http://purl.uniprot.org/uniprot/") AS ?uniprot_id2)
}
```

This query generates a table with columns `uniprot_1` and `uniprot_2`.

Then, We use the networkx library to find the shortest hop count for all pairs:

```python
import networkx as nx
import duckdb
import pandas as pd

rel = duckdb.sql("FROM 'https://pub-419287e3e07041009e8c3eb9f45c8549.r2.dev/layer1/uniprot_protein-protein.parquet'")
df = rel.to_df() # uniprot_id1, uniprot_id2

# Create a graph from the DataFrame
graph = nx.from_pandas_edgelist(df, 'uniprot_id1', 'uniprot_id2')

# Find all pairs of nodes
all_pairs = list(nx.all_pairs_shortest_path_length(graph))

# Extract the data for the DataFrame
data = []
for source, targets in all_pairs:
    for target, distance in targets.items():
        data.append({'source': source, 'target': target, 'distance': distance})

# Create the DataFrame
result_df = pd.DataFrame(data)
```

`result_df` contains the source, target, and distance columns. You can save this DataFrame as a CSV file or something to use it for further analysis.