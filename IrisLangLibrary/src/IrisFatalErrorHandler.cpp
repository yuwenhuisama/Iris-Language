#include "IrisCompileConfigure.h"

#include "IrisFatalErrorHandler.h"
#include "IrisCompiler.h"
#include "IrisInterpreter.h"
#include <sstream>

IrisFatalErrorHandler* IrisFatalErrorHandler::sm_pInstance = nullptr;

IrisFatalErrorHandler * IrisFatalErrorHandler::CurrentFatalHandler()
{
	if (!sm_pInstance) {
		sm_pInstance = new IrisFatalErrorHandler();
	}
	return sm_pInstance;
}

void IrisFatalErrorHandler::SetCurrentFatalHandler(IrisFatalErrorHandler* pHandler)
{
	sm_pInstance = pHandler;
}

void IrisFatalErrorHandler::SetFatalErrorMessageFuncton(FatalErrorMessageFunction pfFunction)
{
	s_pfMessageFunction = pfFunction;
}

void IrisFatalErrorHandler::ShowFatalErrorMessage(FatalErrorType eType, size_t nLineNumber, size_t nBelongingFileIndex, const string & strFatalErrorMessage)
{

	IrisInterpreter::CurrentInterpreter()->HappenFatalError();
	
	string strIrregularTitle = "<Irregular : ";
	string strMessage = "\n  Irregular-happened Report : Oh! Master, a FATAL ERROR has happened and Iris is not clever and dosen't kown how to deal with it. Could you please cheak it yourself? \n";
	string strIrregularMessage = ">The Irregular name is ";
	stringstream ssStream;
	ssStream << nLineNumber;
	string strLinenoMessage = "";
	if (nLineNumber > 0) {
#if IR_USE_STL_STRING
		strLinenoMessage = ">and happened at line " + ssStream.str() + " file " + IrisCompiler::CurrentCompiler()->GetFileName(nBelongingFileIndex) + ".\n";
#else
		strLinenoMessage = ">and happened at line " + ssStream.str() + " file " + IrisCompiler::CurrentCompiler()->GetFileName(nBelongingFileIndex).GetSTLString() + ".\n";
#endif // IR_USE_STL_STRING
	}

	switch (eType)
	{
	case FatalErrorType::SyntaxIrregular:
		strMessage = strIrregularTitle + "SyntaxIrregular>" + strMessage;
		strMessage += strIrregularMessage + "SyntaxIrregular," + "\n" + strLinenoMessage;		
		break;
	case FatalErrorType::NoMethodIrregular:
		strMessage = strIrregularTitle + "NoMethodIrregular>" + strMessage;
		strMessage += strIrregularMessage + "NoMethodIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::IdenfierTypeIrregular:
		strMessage = strIrregularTitle + "IdenfierTypeIrregular>" + strMessage;
		strMessage += strIrregularMessage + "IdenfierTypeIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::IrregularNotDealedIrregular:
		strMessage = strIrregularTitle + "IrregularNotDealedIrregular>" + strMessage;
		strMessage += strIrregularMessage + "IrregularNotDealedIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		{
			auto& ivIrregularObj = IrisDevUtil::GetCurrentThreadInfo()->m_ivIrregularObjectRegister;
			auto nLineNumber = IrisDevUtil::GetInt(IrisDevUtil::GetObjectInstanceVariable(ivIrregularObj, "@line_number"));
			auto szFileName = IrisDevUtil::GetString(IrisDevUtil::GetObjectInstanceVariable(ivIrregularObj, "@file_name"));
			auto szMsg = IrisDevUtil::GetString(IrisDevUtil::GetObjectInstanceVariable(ivIrregularObj, "@message"));
			strMessage += "> file name : ";
			strMessage += szFileName;
			strMessage += "\n";
			strMessage += "> line number : ";
			stringstream ssStream;
			ssStream << nLineNumber;
			strMessage += ssStream.str();
			strMessage += "\n";
			strMessage += "> message : ";
			strMessage += szMsg;
			strMessage += "\n";
		}
		break;
	case FatalErrorType::SourceObjectIrregular:
		strMessage = strIrregularTitle + "SourceObjectIrregular>" + strMessage;
		strMessage += strIrregularMessage + "SourceObjectIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::IdentifierNotFoundIrregular:
		strMessage = strIrregularTitle + "IdentifierNotFoundIrregular>" + strMessage;
		strMessage += strIrregularMessage + "IdentifierNotFoundIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::ReturnIrregular:
		strMessage = strIrregularTitle + "ReturnIrregular>" + strMessage;
		strMessage += strIrregularMessage + "ReturnIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::LeftValueIrregular:
		strMessage = strIrregularTitle + "LeftValueIrregular>" + strMessage;
		strMessage += strIrregularMessage + "LeftValueIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::SelfPointerIrregular:
		strMessage = strIrregularTitle + "SelfPointerIrregular>" + strMessage;
		strMessage += strIrregularMessage + "SelfPointerIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::ConstanceReassignIrregular:
		strMessage = strIrregularTitle + "ConstanceReassignIrregular>" + strMessage;
		strMessage += strIrregularMessage + "ConstanceReassignIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::ConstanceDeclareIrregular:
		strMessage = strIrregularTitle + "ConstanceDeclareIrregular>" + strMessage;
		strMessage += strIrregularMessage + "ConstanceDeclareIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::ClassVariableDeclareIrregular:
		strMessage = strIrregularTitle + "ClassVariableDeclareIrregular>" + strMessage;
		strMessage += strIrregularMessage + "ClassVariableDeclareIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::MethodAuthorityIrregular:
		strMessage = strIrregularTitle + "MethodAuthorityIrregular>" + strMessage;
		strMessage += strIrregularMessage + "MethodAuthorityIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::ClassNotCompleteIrregular:
		strMessage = strIrregularTitle + "ClassNotCompleteIrregular>" + strMessage;
		strMessage += strIrregularMessage + "ClassNotCompleteIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::ParameterNotFitIrregular:
		strMessage = strIrregularTitle + "ParameterNotFitIrregular>" + strMessage;
		strMessage += strIrregularMessage + "ParameterNotFitIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::NoMethodCanSuperIrregular:
		strMessage = strIrregularTitle + "NoMethodCanSuperIrregular>" + strMessage;
		strMessage += strIrregularMessage + "NoMethodCanSuperIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::UnkownIrregular:
		strMessage = strIrregularTitle + "UnkownIrregular>" + strMessage;
		strMessage += strIrregularMessage + "UnkownIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::FieldCannotRoutedIrregular:
		strMessage = strIrregularTitle + "FieldCannotRoutedIrregular>" + strMessage;
		strMessage += strIrregularMessage + "FieldCannotRoutedIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::BreakIrregular:
		strMessage = strIrregularTitle + "BreakIrregular>" + strMessage;
		strMessage += strIrregularMessage + "BreakIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case FatalErrorType::ContinueIrregular:
		strMessage = strIrregularTitle + "ContinueIrregular>" + strMessage;
		strMessage += strIrregularMessage + "ContinueIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case IrisFatalErrorHandler::FatalErrorType::InvalidLeftExpressionIrregular:
		strMessage = strIrregularTitle + "InvalidLeftExpressionIrregular>" + strMessage;
		strMessage += strIrregularMessage + "InvalidLeftExpressionIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case IrisFatalErrorHandler::FatalErrorType::CastExpressionIrregular:
		strMessage = strIrregularTitle + "CastExpressionIrregular>" + strMessage;
		strMessage += strIrregularMessage + "CastExpressionIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case IrisFatalErrorHandler::FatalErrorType::SelfExpressionIrregular:
		strMessage = strIrregularTitle + "SelfExpressionIrregular>" + strMessage;
		strMessage += strIrregularMessage + "SelfExpressionIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case IrisFatalErrorHandler::FatalErrorType::AuthorityStatementIrregular:
		strMessage = strIrregularTitle + "AuthorityStatementIrregular>" + strMessage;
		strMessage += strIrregularMessage + "AuthorityStatementIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case IrisFatalErrorHandler::FatalErrorType::BlockStatementIrregular:
		strMessage = strIrregularTitle + "BlockStatementIrregular>" + strMessage;
		strMessage += strIrregularMessage + "BlockStatementIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case IrisFatalErrorHandler::FatalErrorType::AccessorStatementIrregular:
		strMessage = strIrregularTitle + "AccessorStatementIrregular>" + strMessage;
		strMessage += strIrregularMessage + "AccessorStatementIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case IrisFatalErrorHandler::FatalErrorType::ClassStatementIrregular:
		strMessage = strIrregularTitle + "ClassStatementIrregular>" + strMessage;
		strMessage += strIrregularMessage + "ClassStatementIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case IrisFatalErrorHandler::FatalErrorType::InterfaceFunctionStatementIrregular:
		strMessage = strIrregularTitle + "InterfaceFunctionStatementIrregular>" + strMessage;
		strMessage += strIrregularMessage + "InterfaceFunctionStatementIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	case IrisFatalErrorHandler::FatalErrorType::SuperStatementIrregular:
		strMessage = strIrregularTitle + "SuperStatementIrregular>" + strMessage;
		strMessage += strIrregularMessage + "SuperStatementIrregular," + "\n" + strLinenoMessage;
		strMessage += ">Tip : " + strFatalErrorMessage + "\n";
		break;
	default:
		break;
	}

	s_pfMessageFunction(strMessage);
}

void IrisFatalErrorHandler::ShowFatalErrorDirectly(const string& strFatalErrorMessage)
{
	s_pfMessageFunction(strFatalErrorMessage);
}

IrisFatalErrorHandler::IrisFatalErrorHandler()
{
}

IrisFatalErrorHandler::~IrisFatalErrorHandler()
{
}
