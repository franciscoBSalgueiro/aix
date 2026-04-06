#include "aixchess_functions.hpp"

namespace duckdb {

namespace {

inline void IsValidMovedata(DataChunk &args, ExpressionState &state, Vector &result) {
	UnaryExecutor::Execute<string_t, bool>(args.data[0], result, args.size(), [&](string_t game) {
		diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};
		return Game::is_valid_movedata(data);
	});
}

inline void IsValidMovedataFromFen(DataChunk &args, ExpressionState &state, Vector &result) {
	auto count = args.size();
	result.SetVectorType(VectorType::FLAT_VECTOR);
	auto &validity = FlatVector::Validity(result);

	UnifiedReader<string_t> game_reader;
	UnifiedReader<string_t> fen_reader;
	game_reader.Init(args.data[0], count);
	fen_reader.Init(args.data[1], count);

	auto out = FlatVector::GetData<bool>(result);
	for (idx_t i = 0; i < count; i++) {
		if (game_reader.IsNull(i)) {
			validity.SetInvalid(i);
			continue;
		}

		auto game = game_reader.Get(i);
		diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};

		if (fen_reader.IsNull(i)) {
			out[i] = Game::is_valid_movedata(data);
		} else {
			auto fen = fen_reader.Get(i);
			out[i] = Game::is_valid_movedata_from_fen(data, std::string_view(fen.GetData(), fen.GetSize()));
		}
	}
}

} // namespace

void Register_IsValidMovedata(ExtensionLoader &loader) {
	auto is_valid_movedata_function = ScalarFunction("is_valid_movedata", {LogicalType::BLOB}, LogicalType::BOOLEAN, IsValidMovedata);
	loader.RegisterFunction(is_valid_movedata_function);

	auto is_valid_movedata_from_fen_function =
	    ScalarFunction("is_valid_movedata", {LogicalType::BLOB, LogicalType::VARCHAR}, LogicalType::BOOLEAN, IsValidMovedataFromFen);
	loader.RegisterFunction(is_valid_movedata_from_fen_function);
}

} // namespace duckdb