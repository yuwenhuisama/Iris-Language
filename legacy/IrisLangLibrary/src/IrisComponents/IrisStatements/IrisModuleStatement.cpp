#include "IrisComponents/IrisStatements/IrisModuleStatement.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisUnil/IrisBlock.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisFatalErrorHandler.h"
#include "IrisValidator/IrisExpressionValidateVisitor.h"
#include "IrisValidator/IrisStatementValidateVisitor.h"

#include "IrisThread/IrisThreadManager.h"

bool IrisModuleStatement::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	pMaker->push_env();
	pMaker->cre_cenv(1);

	pCompiler->IncreamDefineIndex();

	if (m_pModuleName->GetType() != IrisIdentifierType::Constance) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pModuleName->GetIdentifierString() + " is not a CONSTANCE.", IrisThreadManager::CurrentThreadManager()->GetMainThreadInfo());
		return false;
	}

	pMaker->def_mld(pCompiler->GetIdentifierIndex(m_pModuleName->GetIdentifierString(), pCompiler->GetCurrentFileIndex()), pCompiler->GetDefineIndex());

	// Modules
	if (m_pModules) {
		if(!m_pModules->Ergodic(
			[&](IrisExpression*& pExpression) -> bool {
			if (!pExpression->Generate()) {
				return false;
			}
			pMaker->add_mld();
			return true;
		}
			))
			return false;;
	}

	// Interfaces
	/*
	if (m_pInterfaces) {
		if(!m_pInterfaces->Ergodic(
			[&](IrisExpression*& pExpression) -> bool {
			if(!pExpression->Generate()) {
				return false;
			}
			pMaker->add_inf();
			return true;
		}
			))
			return false;
	}
	*/
	// Block
	if(m_pBlock) {
		m_pBlock->Generate();
	}
	else {
		pCompiler->IncreamDefineIndex();
		pMaker->blk_def(pCompiler->GetDefineIndex());
		pMaker->end_def(pCompiler->GetDefineIndex());
		pCompiler->DecreamDefineIndex();
	}

	pMaker->end_def(pCompiler->GetDefineIndex());
	pCompiler->DecreamDefineIndex();
	pMaker->pop_env();
	return true;
}

IrisModuleStatement::IrisModuleStatement(IrisIdentifier* pModuleName, IrisList<IrisExpression*>* pModules, IrisList<IrisExpression*>* pInterfaces, IrisBlock* pBlock) : m_pModuleName(pModuleName), m_pModules(pModules), m_pInterfaces(pInterfaces), m_pBlock(pBlock)
{
}


IrisModuleStatement::~IrisModuleStatement()
{
	if (m_pModuleName)
		delete m_pModuleName;
	if (m_pModules) {
		m_pModules->Ergodic([](IrisExpression*& x) -> bool { delete x; x = nullptr; return true; });
		m_pModules->Clear();
		delete m_pModules;
	}
	if (m_pInterfaces) {
		m_pInterfaces->Ergodic([](IrisExpression*& x) -> bool { delete x; x = nullptr; return true; });
		m_pInterfaces->Clear();
		delete m_pInterfaces;
	}
	if (m_pBlock) {
		delete m_pBlock;
	}
}

bool IrisModuleStatement::Validate()
{
	auto pCompiler = IrisCompiler::CurrentCompiler();
	IrisExpressionValidateVisitor ievvExpressionVisitor;
	IrisStatementValidateVisitor isvvStatementVisitor;

	if (!pCompiler->UpperStackEmpty() && pCompiler->GetTopUpperType() != IrisCompiler::UpperType::ModuleBlock) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "module of " + m_pModuleName->GetIdentifierString() + " must be defined in Main environment or Module body.", IrisThreadManager::CurrentThreadManager()->GetMainThreadInfo());
		return false;
	}

	if (m_pModuleName->GetType() != IrisIdentifierType::Constance) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pModuleName->GetIdentifierString() + " is not a CONSTANCE.", IrisThreadManager::CurrentThreadManager()->GetMainThreadInfo());
		return false;
	}

	if (m_pModules && !m_pModules->Ergodic([&](IrisExpression*& pExpression) -> bool {
		if (!pExpression->Accept(&ievvExpressionVisitor)) {
			return false;
		}
		return true;
	})) {
		return false;
	}

	/*
	if (m_pInterfaces && !m_pModules->Ergodic([&](IrisExpression*& pExpression) -> bool {
		if (!pExpression->Accept(&ievvExpressionVisitor)) {
			return false;
		}
		return true;
	})) {
		return false;
	}
	*/

	if (m_pBlock) {
		pCompiler->PushUpperType(IrisCompiler::UpperType::ModuleBlock);

		if (!m_pBlock->Accept(&isvvStatementVisitor)) {
			return false;
		}

		pCompiler->PopUpperType();
	}


	return true;
}
