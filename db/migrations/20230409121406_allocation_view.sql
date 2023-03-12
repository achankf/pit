CREATE VIEW PortfolioAllocationRate AS WITH RECURSIVE ParentTotalWeight AS (
    SELECT
        parent_id,
        sum(weight) AS total_weight
    FROM
        AssetClass
    WHERE
        parent_id IS NOT NULL
    GROUP BY
        parent_id
),
ClassWeight AS (
    SELECT
        a.asset_class_id,
        a.person_id,
        a.parent_id,
        a.class,
        CASE
            WHEN a.parent_id IS NULL THEN 1
            ELSE a.weight / b.total_weight
        END AS class_rate
    FROM
        AssetClass a
        LEFT JOIN ParentTotalWeight b USING(parent_id) -- MUST use left join to grab the NULL entries (base cases)
),
RealRate (
    asset_class_id,
    person_id,
    parent_id,
    class,
    class_rate,
    real_rate
) AS (
    SELECT
        asset_class_id,
        person_id,
        parent_id,
        class,
        class_rate,
        class_rate
    FROM
        ClassWeight
    WHERE
        parent_id IS NULL
    UNION
    ALL
    SELECT
        child.asset_class_id,
        child.person_id,
        child.parent_id,
        child.class,
        child.class_rate,
        child.class_rate * parent.class_rate
    FROM
        ClassWeight child
        INNER JOIN RealRate parent ON child.parent_id = parent.asset_class_id
)
SELECT
    *
FROM
    RealRate
WHERE
    asset_class_id IS NOT NULL
    AND asset_class_id NOT IN (
        SELECT
            parent_id
        FROM
            AssetClass
        WHERE
            parent_id IS NOT NULL
    );

CREATE VIEW PerClassAllocationRate AS WITH TotalWeight AS(
    SELECT
        security_id,
        sum(weight) AS total_weight
    FROM
        AssetAllocation
    GROUP BY
        security_id
)
SELECT
    a.security_id,
    a.asset_class_id,
    SUM(a.weight / b.total_weight) AS rate
FROM
    AssetAllocation a
    INNER JOIN TotalWeight b USING (security_id)
GROUP BY
    a.asset_class_id,
    a.security_id;