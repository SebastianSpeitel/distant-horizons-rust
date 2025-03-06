use duckdb::{CachedStatement, Connection, Result, Row, Statement};

pub trait Query<P = ()> {
    #[inline]
    fn r#where(&self) -> &str {
        "1"
    }

    #[inline]
    fn order_by(&self) -> Option<&str> {
        None
    }

    fn bind_params(&self, stmt: &mut Statement, params: P);

    #[inline]
    fn ordered(self, order: &'static str) -> impl Query<P>
    where
        Self: Sized,
    {
        Ordered {
            order,
            query: self,
            _p: std::marker::PhantomData,
        }
    }
}

impl<F, P> Query<P> for F
where
    F: Fn(&mut Statement, P) + AsRef<str>,
{
    #[inline]
    fn r#where(&self) -> &str {
        self.as_ref()
    }

    #[inline]
    fn bind_params(&self, stmt: &mut Statement, params: P) {
        self(stmt, params);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct All;

impl Query for All {
    #[inline]
    fn r#where(&self) -> &str {
        "1"
    }

    #[inline]
    fn bind_params(&self, _stmt: &mut Statement, _params: ()) {}
}

struct Ordered<P, Q: Query<P>> {
    order: &'static str,
    query: Q,
    _p: std::marker::PhantomData<P>,
}

impl<P, Q: Query<P>> Query<P> for Ordered<P, Q> {
    #[inline]
    fn r#where(&self) -> &str {
        self.query.r#where()
    }

    #[inline]
    fn order_by(&self) -> Option<&str> {
        Some(self.order)
    }

    #[inline]
    fn bind_params(&self, stmt: &mut Statement, params: P) {
        self.query.bind_params(stmt, params);
    }
}

pub trait Repo {
    const SELECTION: &'static str = "*";
    const INSERT: &'static str;
    const TABLE: &'static str;

    type Element<'r>: Sized;

    #[inline]
    fn prepare_select_with<'c, Q: Query<P>, P>(
        conn: &'c Connection,
        query: &Q,
        params: P,
    ) -> Result<CachedStatement<'c>> {
        let sql = format!(
            "SELECT {} FROM {} WHERE {} {}",
            Self::SELECTION,
            Self::TABLE,
            query.r#where(),
            query
                .order_by()
                .map_or_else(String::new, |o| format!("ORDER BY {o}"))
        );
        let mut stmt = conn.prepare_cached(&sql)?;
        query.bind_params(&mut stmt, params);
        stmt.raw_execute()?;
        Ok(stmt)
    }

    #[inline]
    fn select_vec_with<Q: Query<P>, P>(
        conn: &Connection,
        query: &Q,
        params: P,
        into_owned: impl for<'r> Fn(Self::Element<'r>) -> Self::Element<'static>,
    ) -> Result<Vec<Self::Element<'static>>> {
        let stmt = Self::prepare_select_with(conn, query, params)?;

        let mut rows = stmt.raw_query();
        let count = rows.as_ref().map(Statement::row_count).unwrap_or_default();
        let mut elements = Vec::with_capacity(count);

        while let Some(row) = rows.next()? {
            elements.push(into_owned(Self::from_row(row)?));
        }

        Ok(elements)
    }

    #[inline]
    fn select_vec<Q: Query<()>>(
        conn: &Connection,
        query: &Q,
        into_owned: impl for<'r> Fn(Self::Element<'r>) -> Self::Element<'static>,
    ) -> Result<Vec<Self::Element<'static>>> {
        Self::select_vec_with(conn, query, (), into_owned)
    }

    #[inline]
    fn insert(conn: &Connection, element: Self::Element<'_>) -> Result<()> {
        let sql = format!("INSERT INTO {} {}", Self::TABLE, Self::INSERT);
        let mut stmt = conn.prepare_cached(&sql)?;
        Self::bind_insert(&mut stmt, element)?;
        match stmt.raw_execute()? {
            1 => Ok(()),
            c => Err(duckdb::Error::StatementChangedRows(c)),
        }
    }

    fn bind_insert(stmt: &mut Statement, element: Self::Element<'_>) -> Result<()>;

    fn from_row<'r>(row: &'r Row) -> Result<Self::Element<'r>>;
}
