-- Add migration script here
-- 初始化灰度策略类型数据（仅过滤类策略，不包含标签相关策略）

-- 插入过滤类策略
INSERT INTO public.strategy_types (code, name, category, description, processor_type, config_schema, is_active) VALUES
    (
        'filter_whitelist',
        '白名单过滤',
        'filter',
        '基于机器码白名单的灰度策略，只对白名单中的机器生效',
        'filter_whitelist',
        '{
            "type": "object",
            "properties": {
                "machines": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "机器码列表"
                }
            },
            "required": ["machines"]
        }',
        true
    ),
    (
        'filter_percentage',
        '百分比过滤',
        'filter',
        '基于哈希百分比的一致性分配，按百分比随机分配机器到灰度',
        'filter_percentage',
        '{
            "type": "object",
            "properties": {
                "percent": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 100,
                    "description": "百分比 (0-100)"
                }
            },
            "required": ["percent"]
        }',
        true
    ),
    (
        'filter_random',
        '随机过滤',
        'filter',
        '基于随机种子的随机分配策略',
        'filter_random',
        '{
            "type": "object",
            "properties": {
                "seed": {
                    "type": "integer",
                    "description": "随机种子"
                }
            },
            "required": ["seed"]
        }',
        true
    );

