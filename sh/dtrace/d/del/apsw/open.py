#!/usr/local/bin/python3

import apsw

con=apsw.Connection(":memory:")
cur=con.cursor()
for row in cur.execute("create table foo(x,y,z);insert into foo values (?,?,?);"
                       "insert into foo values(?,?,?);select * from foo;drop table foo;"
                       "create table bar(x,y);insert into bar values(?,?);"
                       "insert into bar values(?,?);select * from bar;",
                       (1,2,3,4,5,6,7,8,9,10)):
                           print(row)