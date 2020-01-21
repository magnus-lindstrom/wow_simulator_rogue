import sqlite3


def create_connection():
    """ create a database connection to a SQLite database """
    try:
        connection = sqlite3.connect("../db/items.db")
        return connection
    except Error as e:
        print(e)


def get_agi(conn, name):
    agi_type = '3'

    all_stat_types = ['stat_type{:}'.format(i) for i in range(1, 10)]
    all_values = ['stat_value{:}'.format(i) for i in range(1, 10)]

    for stat_type, value in zip(all_stat_types, all_values):
        cur = conn.cursor()
        search_string = "SELECT {} FROM items WHERE name LIKE ? AND {} LIKE {}".format(
            value, stat_type, agi_type)
        cur.execute(search_string, ('%' + name + '%',))
        rows = cur.fetchall()
        if len(rows) == 0:
            break
        elif len(rows) > 1:
            print("Too many hits for item {}.".format(name))
            exit(1)
        return rows[0][0]


def get_str(conn, name):
    str_type = '4'

    all_stat_types = ['stat_type{:}'.format(i) for i in range(1, 10)]
    all_values = ['stat_value{:}'.format(i) for i in range(1, 10)]

    for stat_type, value in zip(all_stat_types, all_values):
        cur = conn.cursor()
        search_string = "SELECT {} FROM items WHERE name LIKE ? AND {} LIKE {}".format(
            value, stat_type, str_type)
        cur.execute(search_string, ('%' + name + '%',))
        rows = cur.fetchall()
        if len(rows) == 0:
            break
        elif len(rows) > 1:
            print("Too many hits for item {}.".format(name))
            exit(1)
        return rows[0][0]


if __name__ == '__main__':
    conn = create_connection()
    agi = get_agi(conn, 'Shadowcraft Tunic')
    stre = get_str(conn, 'Shadowcraft Tunic')
    print(agi, stre)



