CREATE VIEW ParentTotalWeight AS
SELECT
    parent_id,
    sum(weight) AS total_weight
FROM
    AssetClass
WHERE
    parent_id IS NOT NULL
GROUP BY
    parent_id;

CREATE VIEW ClassWeight AS
SELECT
    asset_class_id,
    person_id,
    parent_id,
    asset_class_name_id,
    CASE
        WHEN parent_id IS NULL THEN 1
        ELSE weight / total_weight
    END AS class_rate
FROM
    AssetClass
    LEFT JOIN ParentTotalWeight USING(parent_id) -- MUST use left join to grab the NULL entries (base cases)
;

CREATE VIEW PortfolioAllocationRate AS WITH RECURSIVE RealRate (
    asset_class_id,
    person_id,
    parent_id,
    asset_class_name_id,
    class_rate,
    real_rate
) AS (
    SELECT
        asset_class_id,
        person_id,
        parent_id,
        asset_class_name_id,
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
        child.asset_class_name_id,
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

-- quick test
SELECT
    *
FROM
    PortfolioAllocationRate;

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
    b.person_id,
    a.security_id,
    b.asset_class_id,
    SUM(a.weight / b.total_weight) AS rate
FROM
    AssetAllocation a
    INNER JOIN AssetClass b USING (asset_class_name_id)
    INNER JOIN TotalWeight b USING (security_id)
GROUP BY
    b.person_id,
    b.asset_class_id,
    a.security_id;

-- quick test
SELECT
    *
FROM
    PerClassAllocationRate;