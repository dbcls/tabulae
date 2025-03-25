FROM 'chembl_compound_ro5'
NATURAL JOIN 'chembl_compound_alogp'
NATURAL JOIN 'chembl_compound_mw'
NATURAL JOIN 'chembl_compound_hba'
NATURAL JOIN 'chembl_compound_hbd';
