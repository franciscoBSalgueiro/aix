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
	BinaryExecutor::Execute<string_t, string_t, bool>(args.data[0], args.data[1], result, args.size(),
	                                                 [&](string_t game, string_t initial_fen) {
		                                                 diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};
		                                                 return Game::is_valid_movedata_from_fen(
		                                                     data,
		                                                     std::string_view(initial_fen.GetData(), initial_fen.GetSize()));
	                                                 });
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