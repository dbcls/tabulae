FROM 'chembl_compound-ro5'
NATURAL JOIN 'chembl_compound-alogp'
NATURAL JOIN 'chembl_compound-mw'
NATURAL JOIN 'chembl_compound-hba'
NATURAL JOIN 'chembl_compound-hbd';