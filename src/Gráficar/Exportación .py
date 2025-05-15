import sqlite3
import pandas as pd

# Ruta del archivo DB
db_path = r'D:\fabia\Documents\metricas-bueno\target\debug\metrics.db'

# Conexión a la base de datos
conn = sqlite3.connect(db_path)

# Leer los datos
df = pd.read_sql_query("SELECT * FROM metrics;", conn)

# Exportar a CSV
csv_path = r'D:\fabia\Documents\metricas-bueno\metrics.csv'
df.to_csv(csv_path, index=False)
print(f"✅ Datos exportados a CSV en: {csv_path}")

# Exportar a JSON
json_path = r'D:\fabia\Documents\metricas-bueno\metrics.json'
df.to_json(json_path, orient='records', lines=True)
print(f"✅ Datos exportados a JSON en: {json_path}")

conn.close()