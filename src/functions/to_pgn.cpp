#include "aixchess_functions.hpp"

namespace duckdb {

namespace {

inline void ToPgn(DataChunk &args, ExpressionState &state, Vector &result) {
	UnaryExecutor::Execute<string_t, string_t>(args.data[0], result, args.size(), [&](string_t game) {
		diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};
		auto pgn_result = Game::to_pgn_string(data);
		auto pgn = UnwrapDecoded<std::string>(std::move(pgn_result), "to_pgn");
		return StringVector::AddString(result, pgn);
	});
}

inline void ToPgnFromFen(DataChunk &args, ExpressionState &state, Vector &result) {
	BinaryExecutor::Execute<string_t, string_t, string_t>(args.data[0], args.data[1], result, args.size(),
	                                                      [&](string_t game, string_t fen) {
		                                                      diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};
		                                                      auto pgn_result = Game::to_pgn_string_from_fen(
		                                                          data, std::string_view(fen.GetData(), fen.GetSize()));
		                                                      auto pgn = UnwrapDecoded<std::string>(std::move(pgn_result), "to_pgn");
		                                                      return StringVector::AddString(result, pgn);
	                                                      });
}

} // namespace

void Register_ToPgn(ExtensionLoader &loader) {
	auto to_pgn_function = ScalarFunction("to_pgn", {LogicalType::BLOB}, LogicalType::VARCHAR, ToPgn);
	loader.RegisterFunction(to_pgn_function);

	auto to_pgn_from_fen_function =
	    ScalarFunction("to_pgn", {LogicalType::BLOB, LogicalType::VARCHAR}, LogicalType::VARCHAR, ToPgnFromFen);
	loader.RegisterFunction(to_pgn_from_fen_function);
}

} // namespace duckdb