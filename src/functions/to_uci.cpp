#include "aixchess_functions.hpp"

namespace duckdb {

namespace {

inline void ToUci(DataChunk &args, ExpressionState &state, Vector &result) {
	UnaryExecutor::Execute<string_t, string_t>(args.data[0], result, args.size(), [&](string_t game) {
		diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};
		auto uci_result = Game::to_uci_string(data);
		auto uci = UnwrapDecoded<std::string>(std::move(uci_result), "to_uci");
		return StringVector::AddString(result, uci);
	});
}

inline void ToUciFromFen(DataChunk &args, ExpressionState &state, Vector &result) {
	BinaryExecutor::Execute<string_t, string_t, string_t>(args.data[0], args.data[1], result, args.size(),
	                                                      [&](string_t game, string_t fen) {
		                                                      diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};
		                                                      auto uci_result = Game::to_uci_string_from_fen(
		                                                          data, std::string_view(fen.GetData(), fen.GetSize()));
		                                                      auto uci = UnwrapDecoded<std::string>(std::move(uci_result), "to_uci");
		                                                      return StringVector::AddString(result, uci);
	                                                      });
}

} // namespace

void Register_ToUci(ExtensionLoader &loader) {
	auto to_uci_function = ScalarFunction("to_uci", {LogicalType::BLOB}, LogicalType::VARCHAR, ToUci);
	loader.RegisterFunction(to_uci_function);

	auto to_uci_from_fen_function =
	    ScalarFunction("to_uci", {LogicalType::BLOB, LogicalType::VARCHAR}, LogicalType::VARCHAR, ToUciFromFen);
	loader.RegisterFunction(to_uci_from_fen_function);
}

} // namespace duckdb