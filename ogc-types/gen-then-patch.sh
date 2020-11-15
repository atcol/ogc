./xgen.sh ./schemas/wms/1.3.0/capabilities_1_3_0.xsd src/wms
patch --input=src/wms.rs.diff ./src/wms.rs

./xgen.sh ./schemas/xlink.xsd src/xlink
patch --input=src/xlink.rs.diff ./src/xlink.rs
