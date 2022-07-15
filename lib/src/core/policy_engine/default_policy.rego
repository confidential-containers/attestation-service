package policy

import future.keywords.every

default allow = false

allow {
	every k, v in input {
		judge_field(k, v)
	}
}

judge_field(input_key, input_value) {
	has_key(data.reference, input_key)
	reference_value := data.reference[input_key]
	match_value(reference_value, input_value)
}

judge_field(input_key, input_value) {
	not has_key(data.reference, input_key)
}

match_value(reference_value, input_value) {
	not is_array(reference_value)
	input_value == reference_value
}

match_value(reference_value, input_value) {
	is_array(reference_value)
	array_include(reference_value, input_value)
}

array_include(reference_value_array, input_value) {
	reference_value_array == []
}

array_include(reference_value_array, input_value) {
	reference_value_array != []
    some i
    reference_value_array[i] == input_value
}

has_key(m, k) {  _ = m[k] }