import sqlite3
import pandas as pd
import os

def export_sqlite_table_relative(table_name='metrics'):
    # Obtener el directorio donde está este script
    base_dir = os.path.dirname(os.path.abspath(__file__))

    # Construir rutas relativas
    db_path = os.path.join(base_dir, 'target', 'debug', 'metrics.db')
    csv_path = os.path.join(base_dir, 'metrics.csv')
    json_path = os.path.join(base_dir, 'metrics.json')

    # Conectar a la base de datos
    conn = sqlite3.connect(db_path)

    # Leer la tabla
    query = f"SELECT * FROM {table_name};"
    df = pd.read_sql_query(query, conn)

    # Exportar a CSV
    df.to_csv(csv_path, index=False)
    print(f"✅ Datos exportados a CSV en: {csv_path}")

    # Exportar a JSON
    df.to_json(json_path, orient='records', lines=True)
    print(f"✅ Datos exportados a JSON en: {json_path}")

    conn.close()

if __name__ == '__main__':
    export_sqlite_table_relative()
